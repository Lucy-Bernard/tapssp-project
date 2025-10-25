//! DIAGNOSIS SERVICE
//!
//! DIAGNOSTIC KERNEL - Core business logic for AI-driven plant diagnosis.
//!
//! This uses a sandbox executor to safely process AI-generated responses.

use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;

use crate::adapters::{AiAdapter, SandboxExecutor, ActionEffect};
use crate::domain::enums::DiagnosisStatus;
use crate::domain::DiagnosisSession;
use crate::dto::{
    DiagnosisAskResponse, DiagnosisConcludeResponse, DiagnosisResponseDto, DiagnosisStartDto,
    DiagnosisUpdateDto,
};
use crate::repositories::{DiagnosisRepository, PlantRepository};

pub struct DiagnosisService {
    plant_repo: PlantRepository,
    diagnosis_repo: DiagnosisRepository,
    ai_adapter: AiAdapter,
    sandbox_executor: SandboxExecutor,
}

impl DiagnosisService {
    pub fn new(
        plant_repo: PlantRepository,
        diagnosis_repo: DiagnosisRepository,
        ai_adapter: AiAdapter,
    ) -> Self {
        Self {
            plant_repo,
            diagnosis_repo,
            ai_adapter,
            sandbox_executor: SandboxExecutor::new(),
        }
    }

    pub async fn start_diagnosis(
        &self,
        plant_id: &str,
        dto: DiagnosisStartDto,
        user_id: String,
    ) -> Result<DiagnosisResponseDto> {
        // Verify plant exists and belongs to user
        let plant = self
            .plant_repo
            .get_by_id(plant_id, &user_id)
            .await?
            .context("Plant not found")?;

        // Create new diagnosis session
        let mut session = DiagnosisSession::new(plant_id.to_string(), dto.prompt.clone());

        // Add plant vitals to context
        if let Some(context) = session.diagnosis_context.as_object_mut() {
            context.insert(
                "plant_vitals".to_string(),
                json!({
                    "name": plant.name,
                    "care_schedule": plant.care_schedule
                }),
            );
        }

        // Save session
        session = self.diagnosis_repo.create(&session).await?;

        // Run diagnosis cycle
        self.run_diagnosis_cycle(session, user_id).await
    }

    pub async fn update_diagnosis(
        &self,
        diagnosis_id: &str,
        dto: DiagnosisUpdateDto,
        user_id: String,
    ) -> Result<DiagnosisResponseDto> {
        // Get existing session
        let mut session = self
            .diagnosis_repo
            .get_by_id(diagnosis_id)
            .await?
            .context("Diagnosis session not found")?;

        // Verify user owns the plant
        let _ = self
            .plant_repo
            .get_by_id(&session.plant_id, &user_id)
            .await?
            .context("Unauthorized access to diagnosis")?;

        // Check status
        if session.status != DiagnosisStatus::PendingUserInput {
            anyhow::bail!("Cannot update a completed or cancelled diagnosis");
        }

        // Append user message to conversation history
        if let Some(context) = session.diagnosis_context.as_object_mut() {
            if let Some(history) = context.get_mut("conversation_history") {
                if let Some(history_array) = history.as_array_mut() {
                    history_array.push(json!({
                        "role": "user",
                        "message": dto.message
                    }));
                }
            }
        }

        // Run diagnosis cycle
        self.run_diagnosis_cycle(session, user_id).await
    }

    pub async fn get_diagnosis(
        &self,
        diagnosis_id: &str,
        user_id: &str,
    ) -> Result<DiagnosisSession> {
        let session = self
            .diagnosis_repo
            .get_by_id(diagnosis_id)
            .await?
            .context("Diagnosis session not found")?;

        // Verify user owns the plant
        let _ = self
            .plant_repo
            .get_by_id(&session.plant_id, user_id)
            .await?
            .context("Unauthorized access to diagnosis")?;

        Ok(session)
    }

    pub async fn delete_diagnosis(&self, diagnosis_id: &str, user_id: &str) -> Result<()> {
        let session = self
            .diagnosis_repo
            .get_by_id(diagnosis_id)
            .await?
            .context("Diagnosis session not found")?;

        // Verify user owns the plant
        let _ = self
            .plant_repo
            .get_by_id(&session.plant_id, user_id)
            .await?
            .context("Unauthorized access to diagnosis")?;

        self.diagnosis_repo.delete(diagnosis_id).await
    }

    pub async fn get_all_by_plant_id(
        &self,
        plant_id: &str,
        user_id: &str,
    ) -> Result<Vec<DiagnosisSession>> {
        // Verify user owns the plant
        let _ = self
            .plant_repo
            .get_by_id(plant_id, user_id)
            .await?
            .context("Plant not found")?;

        self.diagnosis_repo
            .get_all_by_plant_id(plant_id, user_id)
            .await
    }

    async fn run_diagnosis_cycle(
        &self,
        mut session: DiagnosisSession,
        _user_id: String,
    ) -> Result<DiagnosisResponseDto> {
        // Generate AI response for the current diagnosis context
        // The diagnostic prompt is already built into generate_diagnosis_response()
        let ai_response = self
            .ai_adapter
            .generate_diagnosis_response(&session.diagnosis_context)
            .await?;

        // Use sandbox executor to parse and validate the AI response
        let execution_result = self
            .sandbox_executor
            .execute_code(&ai_response, &session.diagnosis_context)
            .await?;

        // Execute the action
        let effect = self
            .sandbox_executor
            .execute_action(&execution_result, &mut session.diagnosis_context)?;

        match effect {
            ActionEffect::Continue => {
                // LOG_STATE was executed, continue with another cycle
                session.updated_at = Utc::now();
                self.diagnosis_repo.update(&session).await?;

                // Recursively run another cycle
                Box::pin(self.run_diagnosis_cycle(session, _user_id)).await
            }
            ActionEffect::FetchPlantVitals => {
                // Should not happen since we populate vitals at start
                // But if it does, fetch and continue
                let plant = self
                    .plant_repo
                    .get_by_id(&session.plant_id, &_user_id)
                    .await?
                    .context("Plant not found")?;

                if let Some(context) = session.diagnosis_context.as_object_mut() {
                    context.insert(
                        "plant_vitals".to_string(),
                        json!({
                            "name": plant.name,
                            "care_schedule": plant.care_schedule
                        }),
                    );
                }

                session.updated_at = Utc::now();
                self.diagnosis_repo.update(&session).await?;

                // Run another cycle with vitals now available
                Box::pin(self.run_diagnosis_cycle(session, _user_id)).await
            }
            ActionEffect::AskUser(question) => {
                // Add AI question to conversation history
                if let Some(context) = session.diagnosis_context.as_object_mut() {
                    if let Some(history) = context.get_mut("conversation_history") {
                        if let Some(history_array) = history.as_array_mut() {
                            history_array.push(json!({
                                "role": "assistant",
                                "message": question.clone()
                            }));
                        }
                    }
                }

                session.status = DiagnosisStatus::PendingUserInput;
                session.updated_at = Utc::now();
                self.diagnosis_repo.update(&session).await?;

                Ok(DiagnosisResponseDto::Ask(DiagnosisAskResponse {
                    diagnosis_id: session.id,
                    question,
                }))
            }
            ActionEffect::Conclude { finding, recommendation } => {
                // Save result to context
                if let Some(context) = session.diagnosis_context.as_object_mut() {
                    context.insert(
                        "result".to_string(),
                        json!({
                            "finding": finding.clone(),
                            "recommendation": recommendation.clone()
                        }),
                    );
                }

                session.status = DiagnosisStatus::Completed;
                session.updated_at = Utc::now();
                self.diagnosis_repo.update(&session).await?;

                Ok(DiagnosisResponseDto::Conclude(DiagnosisConcludeResponse {
                    diagnosis_id: session.id,
                    finding,
                    recommendation,
                }))
            }
        }
    }
}