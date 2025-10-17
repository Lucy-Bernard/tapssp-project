# Diagnosis Endpoint Architecture

## 🎯 Overview

This document explains how the diagnosis endpoint works from top to bottom, with a focus on Rust concepts like **ownership**, **borrowing**, **generics**, and **traits**.

## 📊 Architecture Layers

The application follows a clean layered architecture:

```
┌─────────────────────────────────────────┐
│         CLI Layer (diagnose.rs)         │  ← User interface
├─────────────────────────────────────────┤
│      Service Layer (routes.rs)          │  ← Business logic orchestration
├─────────────────────────────────────────┤
│  Engine Layer (executor.rs)              │  ← AI/kernel logic
│  Database Layer (repositories)          │  ← Data persistence
├─────────────────────────────────────────┤
│       Models Layer (models/)             │  ← Data structures
└─────────────────────────────────────────┘
```

## 🔄 Request Flow

When you run: `cargo run -- diagnose --plant-id <UUID> --prompt "leaves browning"`

### Step 1: CLI Parsing (main.rs)
```rust
let cli = Cli::parse();  // clap parses command-line arguments
```
- **What happens**: The `clap` library reads your command and matches it to the `Commands::Diagnose` variant
- **Rust concept**: Pattern matching on enums

### Step 2: Database Initialization
```rust
let conn = Arc::new(Mutex::new(Connection::open("plant_id.db")?));
```
- **What happens**: Opens SQLite database and wraps it for thread-safe sharing
- **Rust concepts**:
  - `Arc<T>` - Atomic Reference Counting (multiple owners)
  - `Mutex<T>` - Mutual exclusion (thread-safe access)
  - `?` operator - Early return on error

### Step 3: Dependency Injection
```rust
let plant_repo = PlantRepository::new(Arc::clone(&conn));
let diagnosis_repo = DiagnosisRepository::new(Arc::clone(&conn));
let kernel = KernelExecutor::new();
let diagnosis_service = DiagnosisService::new(plant_repo, diagnosis_repo, kernel);
```
- **What happens**: We build all components and wire them together
- **Rust concepts**:
  - **Ownership**: Components are moved into the service
  - **Borrowing**: Arc is cloned to share the connection
  - **Generics**: Repositories use generic `Result<T, E>` types

### Step 4: Call CLI Handler
```rust
cli::run_diagnose(&diagnosis_service, plant_uuid, prompt).await?;
```
- **What happens**: Delegates to the CLI module
- **Rust concepts**:
  - **Borrowing**: `&diagnosis_service` borrows (doesn't take ownership)
  - **Async/await**: The function is async, so we await it

### Step 5: Service Layer Processing (routes.rs)
```rust
pub async fn create_diagnosis(
    &self,           // Borrows the service
    plant_id: Uuid,  // Owns the UUID
    prompt: String,  // Owns the String
) -> Result<(Uuid, String, DiagnosisStatus)>
```

The service executes 5 sub-steps:

#### 5a. Validate Plant Exists
```rust
let plant = self.plant_repo.get_by_id(plant_id)
    .map_err(|e| anyhow!("Database error: {}", e))?;
```
- **Rust concepts**:
  - **Borrowing**: `self.plant_repo` borrows the repository
  - **Result handling**: `map_err` transforms the error type
  - **Trait**: Using the `?` operator requires `Result` trait

#### 5b. Create Diagnosis Session
```rust
let mut session = DiagnosisSession {
    id: diagnosis_id,
    plant_id,
    status: DiagnosisStatus::Processing,
    // ...
};
```
- **Rust concepts**:
  - **Ownership**: We own this new struct
  - **Mutability**: Marked `mut` because kernel will modify it

#### 5c. Run Kernel Cycle
```rust
let ai_question = self.kernel.run_initial_cycle(&mut session).await?;
```
- **Rust concepts**:
  - **Mutable borrowing**: `&mut session` allows kernel to modify
  - **Async**: Await the asynchronous operation

#### 5d. Persist to Database
```rust
self.diagnosis_repo.create(&session)
```
- **Rust concepts**:
  - **Borrowing**: `&session` borrows (doesn't move ownership)
  - **Trait**: `serde::Serialize` trait enables JSON conversion

#### 5e. Return Results
```rust
Ok((diagnosis_id, ai_question, session.status))
```
- **Rust concepts**:
  - **Tuple**: Returning multiple values
  - **Move semantics**: Values are moved to caller

### Step 6: Display Results (diagnose.rs)
```rust
println!("✅ Diagnosis session created!");
println!("   Diagnosis ID: {}", diagnosis_id);
```
- **What happens**: Format and display the results to the user
