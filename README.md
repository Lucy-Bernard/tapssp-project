# Plant Care CLI - AI-Powered Plant Diagnosis System

## 📋 Overview

A command-line tool that provides an interactive, AI-driven plant diagnosis system. The core innovation is a **Diagnostic Engine** that executes a cyclical reasoning loop where an LLM generates decision logic as executable Rust code snippets, which are compiled and executed in real-time within a sandboxed environment.

## 🎯 Systems Focus Areas

### 1. Performance & Concurrency
- **Async I/O** for API calls using `tokio`
- **Parallel processing** of multiple diagnoses
- **Efficient memory management** with Rust's ownership system

### 2. Security & Sandboxing
- **Safe code execution** using `wasmer` (WebAssembly runtime) or restricted Rust compilation
- **Isolating LLM-generated logic** from system resources

### 3. Data Management
- **Local SQLite database** for plant data and diagnosis history
- **Efficient serialization** with `serde`
- **Optional S3-compatible storage** for images

### 4. LLM Integration
- **HTTP client** for OpenRouter.ai API
- **Streaming responses** for real-time interaction

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│     Controller Layer (endpoints)        │  ← CLI commands & user interaction
├─────────────────────────────────────────┤
│     Service Layer (business logic)      │  ← DiagnosisService, PlantService
├─────────────────────────────────────────┤
│     Engine Layer (AI logic)              │  ← KernelExecutor
│     Repository Layer (data access)      │  ← Database operations
├─────────────────────────────────────────┤
│     Models Layer (data structures)       │  ← Data models & types
└─────────────────────────────────────────┘
```

## 📁 Project Structure

```
plant_id/
├── src/
│   ├── controller/              # CLI endpoints
│   │   ├── diagnosis_controller.rs
│   │   └── plant_controller.rs
│   ├── service/                 # Business logic services
│   │   ├── routes.rs           # DiagnosisService
│   │   ├── openrouter.rs       # OpenRouter API client
│   │   └── plant_id.rs         # PlantID API client
│   ├── engine/                  # AI diagnosis engine
│   │   └── executor.rs         # KernelExecutor
│   ├── database/                # Data persistence
│   │   └── repository/
│   │       ├── diagnosis_repository.rs
│   │       └── plant_repository.rs
│   ├── models/                  # Data structures
│   │   ├── diagnosis.rs
│   │   ├── plant.rs
│   │   ├── chat.rs
│   │   └── common.rs
│   ├── main.rs                  # Application entry point
│   └── lib.rs                   # Library exports
├── examples/
│   └── setup_test_data.rs      # Test data setup
└── Cargo.toml                   # Dependencies
```

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.70+ (with `cargo`)
- **SQLite** (bundled via rusqlite)

### Installation

```bash
# Clone the repository
cd tapssp-project/plant_id

# Build the project
cargo build --release
```

### Setup Test Data

```bash
# Create a test plant in the database
cargo run --example setup_test_data
```

## 💻 Usage

### Available Commands

#### Diagnose a Plant Problem

```bash
cargo run -- diagnose --plant-id <UUID> --prompt "The leaves are browning and crispy."
```

**Example Output:**
```
🔍 Starting diagnosis for plant c0283166-db43-4787-9f13-a81745438b19...

✅ Diagnosis session created!
   Diagnosis ID: 969b11fd-a55c-492e-b6ed-29f4c7ab1401
   Status: PENDING_USER_INPUT

🤖 AI Response:
   Can you tell me how often you've been watering this plant?
```

#### View Diagnosis Session

```bash
cargo run -- show-diagnosis --diagnosis-id <UUID>
```

**Example Output:**
```
📋 Diagnosis Session: 969b11fd-a55c-492e-b6ed-29f4c7ab1401
   Plant ID: c0283166-db43-4787-9f13-a81745438b19
   Status: PendingUserInput
   Created: 2025-10-17 21:57:15 UTC
   Updated: 2025-10-17 21:57:15 UTC

💬 Conversation:
   👤 User [21:57:15]: The leaves are browning and crispy.
   🤖 AI [21:57:15]: Can you tell me how often you've been watering this plant?
```

#### Add a Plant (Coming Soon)

```bash
cargo run -- add --image /path/to/plant.jpg
```

#### List All Plants (Coming Soon)

```bash
cargo run -- list
```

## 🛠️ Technical Implementation

### Diagnosis Flow

1. **User submits problem description** via CLI
2. **Service validates** plant exists in database
3. **Kernel executor** generates AI response using rule-based logic (OpenRouter integration planned)
4. **Session persisted** to SQLite with conversation history
5. **Results displayed** to user with diagnosis ID for follow-up

## 📦 Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }      # CLI parsing
tokio = { version = "1.40", features = ["full"] }      # Async runtime
reqwest = { version = "0.12", features = ["json"] }    # HTTP client
serde = { version = "1.0", features = ["derive"] }     # Serialization
serde_json = "1.0"                                     # JSON handling
rusqlite = { version = "0.32", features = ["bundled"] } # SQLite
uuid = { version = "1.10", features = ["v4", "serde"] } # UUID generation
chrono = { version = "0.4", features = ["serde"] }     # Date/time
anyhow = "1.0"                                         # Error handling
thiserror = "1.0"                                      # Custom errors
base64 = "0.22"                                        # Encoding
image = "0.25"                                         # Image processing
```

## 🗄️ Database Schema

### Plants Table
```sql
CREATE TABLE plants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    care_instructions TEXT,
    watering_schedule TEXT,
    image_path TEXT,
    created_at TEXT NOT NULL
);
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
);
```

## 🔮 Roadmap

### Phase 1: Core Diagnosis (Current)
- [x] Basic CLI structure
- [x] Diagnosis session creation
- [x] Rule-based AI responses
- [x] SQLite persistence
- [x] Repository pattern implementation

### Phase 2: AI Integration (Planned)
- [ ] OpenRouter API client
- [ ] Streaming LLM responses
- [ ] Context-aware follow-up questions
- [ ] Complete diagnosis with recommendations

### Phase 3: Enhanced Features (Future)
- [ ] Plant identification via image
- [ ] Plant care reminders
- [ ] Export diagnosis reports
- [ ] TUI (Terminal UI) with `ratatui`

### Phase 4: Advanced Systems (Future)
- [ ] WebAssembly sandbox for LLM-generated code
- [ ] Parallel diagnosis processing
- [ ] S3 image storage integration
- [ ] Real-time streaming updates

## 📚 Documentation

- [Architecture Guide](ARCHITECTURE_EXPLAINED.md) - Detailed explanation of Rust concepts and design patterns
- [Diagnosis Endpoint](DIAGNOSIS_ENDPOINT.md) - Implementation details for the diagnosis feature

## 🙏 Acknowledgments

- Built with Rust 🦀
- Uses OpenRouter.ai for LLM access
- Inspired by real-world plant care challenges

