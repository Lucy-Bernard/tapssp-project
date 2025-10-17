# Plant ID - Diagnosis Endpoint

## Overview

This CLI tool provides a diagnosis endpoint that creates a new diagnosis session and runs the first cycle of the Kernel loop to help identify plant problems.

## Architecture

The implementation follows Rust best practices with:

- **Ownership & Borrowing**: The `DiagnosisService` uses `Arc<Mutex<Connection>>` for safe concurrent database access
- **Traits**: Leverages `serde::Serialize/Deserialize` for data serialization
- **Generics**: Uses generic error handling with `Result<T, E>` and `anyhow::Result<T>`
- **API Design**: Clean separation of concerns across layers:
  - **Models**: Data structures (`DiagnosisSession`, `Plant`, etc.)
  - **Database**: Repository pattern for data access (`PlantRepository`, `DiagnosisRepository`)
  - **Engine**: Business logic (`KernelExecutor`)
  - **API**: Service layer (`DiagnosisService`)
  - **CLI**: User interface commands

## Features

### Diagnosis Command

Creates a new diagnosis session for a plant and returns the AI's first question:

```bash
cargo run -- diagnose --plant-id <UUID> --prompt "The leaves are browning and crispy."
```

**Output:**
```
🔍 Starting diagnosis for plant c0283166-db43-4787-9f13-a81745438b19...

✅ Diagnosis session created!
   Diagnosis ID: 59c84ff6-2341-4797-8f76-f872ae06d5ac
   Status: PENDING_USER_INPUT

🤖 AI Response:
   To begin, are the brown spots mostly on the leaves getting the most sun?
```

### Show Diagnosis Command

Displays the details of a diagnosis session including conversation history:

```bash
cargo run -- show-diagnosis --diagnosis-id <UUID>
```

**Output:**
```
📋 Diagnosis Session: 59c84ff6-2341-4797-8f76-f872ae06d5ac
   Plant ID: c0283166-db43-4787-9f13-a81745438b19
   Status: PendingUserInput
   Created: 2025-10-17 21:57:15.840471 UTC
   Updated: 2025-10-17 21:57:15.840474 UTC

💬 Conversation:
   👤 User [21:57:15]: The leaves are browning and crispy.
   🤖 AI [21:57:15]: To begin, are the brown spots mostly on the leaves getting the most sun?
```

## Implementation Details

### DiagnosisService

The service layer handles the diagnosis workflow:

1. **Validates** that the plant exists in the database
2. **Creates** a new diagnosis session with a unique ID
3. **Executes** the kernel's first cycle to generate an AI response
4. **Persists** the session to the database
5. **Returns** the diagnosis ID, AI question, and status

### KernelExecutor

The kernel implements a simple rule-based system that:

- Analyzes the user's initial prompt
- Generates contextual follow-up questions based on keywords
- Maintains conversation history
- Updates session status

### Error Handling

The implementation uses Rust's `Result` type and the `anyhow` crate for comprehensive error handling:

- `404` equivalent: "Plant with ID {plant_id} not found"
- Database errors are propagated with context
- All errors include descriptive messages

## Database Schema

### Plants Table
```sql
CREATE TABLE plants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    care_instructions TEXT,
    watering_schedule TEXT,
    image_path TEXT,
    created_at TEXT NOT NULL
)
```

### Diagnoses Table
```sql
CREATE TABLE diagnoses (
    id TEXT PRIMARY KEY,
    plant_id TEXT NOT NULL,
    status TEXT NOT NULL,
    context TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(plant_id) REFERENCES plants(id)
)
```

## Testing

To test the diagnosis endpoint:

1. **Create test data:**
   ```bash
   cargo run --example setup_test_data
   ```

2. **Run a diagnosis:**
   ```bash
   cargo run -- diagnose --plant-id <UUID> --prompt "Your problem description"
   ```

3. **View the diagnosis:**
   ```bash
   cargo run -- show-diagnosis --diagnosis-id <UUID>
   ```

## Future Enhancements

- Integration with OpenRouter API for real AI responses
- Continue diagnosis conversation with follow-up answers
- Complete diagnosis with final recommendations
- Export diagnosis reports
- TUI interface with `ratatui` for interactive diagnosis sessions

