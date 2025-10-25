/*
 * CLI COMMAND IMPLEMENTATIONS
 *
 * Implements the actual command handlers that interact with services.
 * These are the primary adapters that translate CLI user input into
 * domain service calls.
 */

use anyhow::{Context, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;

use crate::adapters::{AiAdapter, PlantIdAdapter, StorageAdapter};
use crate::config::Database;
use crate::domain::enums::DiagnosisStatus;
use crate::dto::{DiagnosisStartDto, DiagnosisUpdateDto, PlantCreationDto};
use crate::repositories::{DiagnosisRepository, PlantRepository};
use crate::services::{DiagnosisService, PlantService};

pub async fn add_plant(
    db: Database,
    image_path: String,
    _name: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
) -> Result<()> {
    println!("{}", style("🌱 Adding new plant...").green().bold());

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("Reading image file...");

    // Read and encode image
    let image_path = Path::new(&image_path);
    if !image_path.exists() {
        anyhow::bail!("Image file not found: {}", image_path.display());
    }

    let image_bytes = fs::read(image_path)
        .context("Failed to read image file")?;
    let base64_image = base64::encode(&image_bytes);

    // Initialize services
    let plant_id_adapter = PlantIdAdapter::new()?;
    let ai_adapter = AiAdapter::new()?;
    let storage_adapter = StorageAdapter::new();
    let plant_repo = PlantRepository::new(db.clone());
    let plant_service = PlantService::new(
        plant_repo,
        plant_id_adapter,
        ai_adapter,
        storage_adapter,
    );

    spinner.set_message("Identifying plant...");

    let dto = PlantCreationDto {
        images: vec![base64_image],
        latitude,
        longitude,
    };

    let plant = plant_service.create_plant(dto, "local-user".to_string()).await?;

    spinner.finish_and_clear();

    println!("{}", style("✓ Plant added successfully!").green().bold());
    println!("\n{}", style("Plant Details:").cyan().bold());
    println!("  {} {}", style("ID:").dim(), plant.id);
    println!("  {} {}", style("Name:").dim(), plant.name);
    println!("\n{}", style("Care Schedule:").cyan().bold());
    println!("  {} {}", style("Light:").dim(), plant.care_schedule.light);
    println!("  {} {}", style("Water:").dim(), plant.care_schedule.water);
    println!("  {} {}", style("Humidity:").dim(), plant.care_schedule.humidity);
    println!("  {} {}", style("Temperature:").dim(), plant.care_schedule.temperature);

    Ok(())
}

pub async fn list_plants(db: Database) -> Result<()> {
    let plant_repo = PlantRepository::new(db);
    let plants = plant_repo.get_all_by_user("local-user").await?;

    if plants.is_empty() {
        println!("{}", style("No plants in your collection yet.").yellow());
        println!("Use {} to add your first plant!", style("plant-care add --image <path>").green());
        return Ok(());
    }

    println!("{}", style(format!("🌿 Your Plant Collection ({} plants)", plants.len())).green().bold());
    println!();

    for plant in plants {
        println!("{}", style(&plant.name).cyan().bold());
        println!("  {} {}", style("ID:").dim(), plant.id);
        println!("  {} {}", style("Added:").dim(), plant.created_at.format("%Y-%m-%d"));
        println!();
    }

    Ok(())
}

pub async fn show_plant(db: Database, plant_identifier: String) -> Result<()> {
    let plant_repo = PlantRepository::new(db);

    // Try to find plant by ID or name
    let plant = plant_repo
        .get_by_id(&plant_identifier, "local-user")
        .await?
        .or_else(|| {
            // TODO: Search by name
            None
        })
        .context("Plant not found")?;

    println!("{}", style(&plant.name).green().bold());
    println!("\n{}", style("Details:").cyan().bold());
    println!("  {} {}", style("ID:").dim(), plant.id);
    println!("  {} {}", style("Added:").dim(), plant.created_at.format("%Y-%m-%d %H:%M"));

    if let Some(url) = &plant.image_url {
        println!("  {} {}", style("Image:").dim(), url);
    }

    println!("\n{}", style("Care Schedule:").cyan().bold());
    println!("  {} {}", style("Light:").dim(), plant.care_schedule.light);
    println!("  {} {}", style("Water:").dim(), plant.care_schedule.water);
    println!("  {} {}", style("Humidity:").dim(), plant.care_schedule.humidity);
    println!("  {} {}", style("Temperature:").dim(), plant.care_schedule.temperature);

    if !plant.care_schedule.care_instructions.is_empty() {
        println!("\n{}", style("Care Instructions:").cyan().bold());
        println!("  {}", plant.care_schedule.care_instructions);
    }

    Ok(())
}

pub async fn delete_plant(db: Database, plant_identifier: String) -> Result<()> {
    let plant_repo = PlantRepository::new(db);
    plant_repo.delete(&plant_identifier, "local-user").await?;

    println!("{}", style("✓ Plant deleted successfully").green().bold());

    Ok(())
}

pub async fn diagnose_plant(
    db: Database,
    plant_identifier: String,
    problem: String,
) -> Result<()> {
    println!("{}", style("🔍 Starting diagnostic session...").green().bold());
    println!();

    // Initialize services
    let plant_repo = PlantRepository::new(db.clone());
    let diagnosis_repo = DiagnosisRepository::new(db.clone());
    let ai_adapter = AiAdapter::new()?;

    let diagnosis_service = DiagnosisService::new(
        plant_repo.clone(),
        diagnosis_repo.clone(),
        ai_adapter,
    );

    // Find plant
    let plant = plant_repo
        .get_by_id(&plant_identifier, "local-user")
        .await?
        .context("Plant not found")?;

    println!("Diagnosing: {}", style(&plant.name).cyan().bold());
    println!("Problem: {}", style(&problem).yellow());
    println!();

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("AI is analyzing...");

    // Start diagnosis
    let dto = DiagnosisStartDto { prompt: problem };
    let response = diagnosis_service
        .start_diagnosis(&plant.id, dto, "local-user".to_string())
        .await?;

    spinner.finish_and_clear();

    // Interactive loop
    match response {
        crate::dto::DiagnosisResponseDto::Ask(ask_response) => {
            let mut diagnosis_id = ask_response.diagnosis_id;
            let mut question = ask_response.question;

            loop {
                println!("{} {}", style("AI:").cyan().bold(), question);

                let answer: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("You")
                    .interact_text()?;

                let spinner = ProgressBar::new_spinner();
                spinner.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.green} {msg}")
                        .unwrap(),
                );
                spinner.set_message("AI is thinking...");

                let update_dto = DiagnosisUpdateDto { message: answer };
                let response = diagnosis_service
                    .update_diagnosis(&diagnosis_id, update_dto, "local-user".to_string())
                    .await?;

                spinner.finish_and_clear();

                match response {
                    crate::dto::DiagnosisResponseDto::Ask(ask_response) => {
                        diagnosis_id = ask_response.diagnosis_id;
                        question = ask_response.question;
                    }
                    crate::dto::DiagnosisResponseDto::Conclude(conclude_response) => {
                        println!();
                        println!("{}", style("🎯 Diagnosis Complete!").green().bold());
                        println!();
                        println!("{}", style("Finding:").cyan().bold());
                        println!("  {}", conclude_response.finding);
                        println!();
                        println!("{}", style("Recommendation:").cyan().bold());
                        println!("  {}", conclude_response.recommendation);
                        break;
                    }
                }
            }
        }
        crate::dto::DiagnosisResponseDto::Conclude(conclude_response) => {
            println!("{}", style("🎯 Diagnosis Complete!").green().bold());
            println!();
            println!("{}", style("Finding:").cyan().bold());
            println!("  {}", conclude_response.finding);
            println!();
            println!("{}", style("Recommendation:").cyan().bold());
            println!("  {}", conclude_response.recommendation);
        }
    }

    Ok(())
}

pub async fn show_history(db: Database, plant_identifier: String) -> Result<()> {
    let plant_repo = PlantRepository::new(db.clone());
    let diagnosis_repo = DiagnosisRepository::new(db);

    let plant = plant_repo
        .get_by_id(&plant_identifier, "local-user")
        .await?
        .context("Plant not found")?;

    let sessions = diagnosis_repo
        .get_all_by_plant_id(&plant.id, "local-user")
        .await?;

    if sessions.is_empty() {
        println!("{}", style("No diagnosis history for this plant.").yellow());
        return Ok(());
    }

    println!(
        "{}",
        style(format!("📋 Diagnosis History for {} ({} sessions)", plant.name, sessions.len()))
            .green()
            .bold()
    );
    println!();

    for session in sessions {
        println!("{}", style(&session.id).cyan());
        println!("  {} {:?}", style("Status:").dim(), session.status);
        println!("  {} {}", style("Created:").dim(), session.created_at.format("%Y-%m-%d %H:%M"));

        if session.status == DiagnosisStatus::Completed {
            if let Some(result) = session.diagnosis_context.get("result") {
                println!("  {} {}", style("Finding:").dim(), result.get("finding").and_then(|v| v.as_str()).unwrap_or("N/A"));
            }
        }
        println!();
    }

    Ok(())
}

pub async fn generate_care(plant_name: String) -> Result<()> {
    println!("{}", style(format!("🌿 Generating care schedule for {}...", plant_name)).green().bold());

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("Consulting AI...");

    let ai_adapter = AiAdapter::new()?;
    let care_schedule = ai_adapter.generate_care_schedule(&plant_name).await?;

    spinner.finish_and_clear();

    println!();
    println!("{}", style("Care Schedule:").cyan().bold());
    println!("  {} {}", style("Light:").dim(), care_schedule.light);
    println!("  {} {}", style("Water:").dim(), care_schedule.water);
    println!("  {} {}", style("Humidity:").dim(), care_schedule.humidity);
    println!("  {} {}", style("Temperature:").dim(), care_schedule.temperature);

    if !care_schedule.care_instructions.is_empty() {
        println!("\n{}", style("Care Instructions:").cyan().bold());
        println!("  {}", care_schedule.care_instructions);
    }

    Ok(())
}
