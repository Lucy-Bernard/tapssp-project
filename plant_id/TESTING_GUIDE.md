# Plant Care CLI - Testing Guide

## 📋 Prerequisites

Before testing the diagnostic kernel, ensure you have:

1. **Rust installed** (version 1.70 or higher)
   ```bash
   rustc --version
   ```

2. **Required API Keys** in your `.env` file:
   ```bash
   OPENROUTER_API_KEY=your_openrouter_key_here
   PLANT_ID_API_KEY=your_plantid_key_here
   DATABASE_PATH=./data/plant_care.db
   ```

3. **Database directory** created:
   ```bash
   mkdir -p data
   ```

4. **A plant image** for testing (e.g., in `src/plant_images/`)

---

## 🚀 Step-by-Step Testing Guide

### Step 1: Build the Application

```bash
cd /Users/lucy/Downloads/tapssp-project/plant_id
cargo build --release
```

This compiles the application in release mode for optimal performance.

---

### Step 2: Initialize the Database

The database will be automatically created when you run your first command. To verify:

```bash
cargo run -- list
```

You should see: `"No plants in your collection yet."`

---

### Step 3: Add a Plant to Your Collection

Before you can diagnose a plant, you need to add it to your collection:

```bash
cargo run -- add --image src/plant_images/fern.png
```

**What happens:**
1. 🔍 The CLI reads your image file
2. 📤 Sends it to Plant.id API for identification
3. 🤖 Uses OpenRouter AI to generate a care schedule
4. 💾 Saves the plant to your local SQLite database

**Expected output:**
```
🌱 Adding new plant...
✓ Plant added successfully!

Plant Details:
  ID: e7d9624b-dabf-4197-b970-84777d9a2592
  Name: Polystichum acrostichoides

Care Schedule:
  Light: Partial to full shade...
  Water: Keep soil consistently moist...
  Humidity: Prefers high humidity...
  Temperature: Thrives in temperatures...
```

**⚠️ Important:** Copy the Plant ID - you'll need it for diagnosis!

---

### Step 4: List All Plants

Verify your plant was added:

```bash
cargo run -- list
```

**Expected output:**
```
🌿 Your Plant Collection (1 plants)

Polystichum acrostichoides
  ID: e7d9624b-dabf-4197-b970-84777d9a2592
  Added: 2025-10-25
```

---

### Step 5: Test the Diagnostic Kernel 🔬

Now for the main feature! Start a diagnosis session:

```bash
cargo run -- diagnose <PLANT_ID> --problem "My fern leaves are browning at the tips"
```

Replace `<PLANT_ID>` with your plant's ID from Step 3.

**Example:**
```bash
cargo run -- diagnose e7d9624b-dabf-4197-b970-84777d9a2592 --problem "My fern leaves are browning at the tips"
```

---

### Step 6: Interactive Diagnosis Conversation

The diagnostic kernel will start an interactive conversation:

```
🔍 Starting diagnostic session...

Diagnosing: Polystichum acrostichoides
Problem: My fern leaves are browning at the tips

AI: How often are you watering your fern, and does the soil feel consistently moist or does it dry out between waterings?
You: I water it every day and the soil is always wet
```

**The AI Diagnostic Kernel Process:**

1. **Initial Analysis**: The AI receives:
   - Your problem description
   - Plant vitals (name, care requirements)
   - Conversation history

2. **Python Code Generation**: The AI generates Python code to determine the next action:
   - `ASK_USER`: Ask a follow-up question
   - `GET_PLANT_VITALS`: Fetch more plant data
   - `LOG_STATE`: Save hypothesis/reasoning
   - `CONCLUDE`: Provide final diagnosis

3. **Sandbox Execution**: The generated Python code runs safely in a sandbox

4. **Action Taken**: Based on the code output, the system either:
   - Asks you another question
   - Provides the final diagnosis

5. **Iterative Process**: Steps 1-4 repeat until diagnosis is complete

---

### Step 7: Receive Final Diagnosis

After answering several questions, you'll receive:

```
🎯 Diagnosis Complete!

Finding:
  Overwatering and Root Rot

Recommendation:
  Reduce watering frequency to once per week. Allow the top 2 inches of soil to dry out between waterings. Check for root rot by gently removing the plant and inspecting roots. If roots are brown and mushy, trim affected areas and repot in fresh, well-draining soil.
```

---

## 🧪 Test Scenarios

### Test Case 1: Watering Issues
```bash
Problem: "Leaves are turning yellow and drooping"
Expected AI Questions:
- How often do you water?
- Is the soil wet or dry?
- Are the yellow leaves at the top or bottom?
```

### Test Case 2: Light Issues
```bash
Problem: "My plant is growing very slowly and looks pale"
Expected AI Questions:
- How much light does it receive daily?
- What direction does the window face?
- Are the leaves pale green or yellow?
```

### Test Case 3: Pest Issues
```bash
Problem: "I see small white spots on the leaves"
Expected AI Questions:
- Are the spots fuzzy or hard?
- Do you see any insects?
- Are the spots only on top of leaves or underneath too?
```

---

## 📊 View Diagnosis History

Check past diagnosis sessions for a plant:

```bash
cargo run -- history <PLANT_ID>
```

**Example output:**
```
📋 Diagnosis History for Polystichum acrostichoides (2 sessions)

diag_abc123
  Status: Completed
  Created: 2025-10-25 14:30
  Finding: Overwatering

diag_xyz789
  Status: PendingUserInput
  Created: 2025-10-25 15:45
```

---

## 🔍 View Plant Details

See full information about a specific plant:

```bash
cargo run -- show <PLANT_ID>
```

**Example output:**
```
Polystichum acrostichoides

Details:
  ID: e7d9624b-dabf-4197-b970-84777d9a2592
  Added: 2025-10-25 14:00
  Image: ./data/plants/e7d9624b.png

Care Schedule:
  Light: Partial to full shade...
  Water: Keep soil consistently moist...
  Humidity: Prefers high humidity...
  Temperature: Thrives in temperatures...
```

---

## 🗑️ Delete a Plant

Remove a plant from your collection:

```bash
cargo run -- delete <PLANT_ID>
```

---

## 💡 Generate Care Schedule (Without Adding Plant)

Get care instructions for any plant without adding it to your collection:

```bash
cargo run -- care "Monstera Deliciosa"
```

**Example output:**
```
🌿 Generating care schedule for Monstera Deliciosa...

Care Schedule:
  Light: Bright, indirect light...
  Water: Water when top 2-3 inches...
  Humidity: 60-80% humidity ideal...
  Temperature: 65-85°F (18-29°C)...
```

---

## ⚠️ Important Notes

### API Credit Usage

**OpenRouter API** (for AI diagnosis):
- Each diagnosis conversation uses ~2-5 credits per question/answer cycle
- A typical diagnosis session (3-5 questions) = ~6-15 credits
- Monitor your usage at: https://openrouter.ai/

**Plant.id API** (for plant identification):
- Each plant identification = 1 credit
- 100 credits = 100 plant identifications
- Monitor your usage at: https://web.plant.id/

### Cost Optimization Tips

1. **Be specific with initial problem descriptions** - helps AI diagnose faster
2. **Answer questions thoroughly** - reduces back-and-forth
3. **Test with `cargo run -- care <name>`** first - doesn't use Plant.id credits

---

## 🐛 Troubleshooting

### Error: "unable to open database file"
```bash
mkdir -p data
echo "DATABASE_PATH=./data/plant_care.db" >> .env
```

### Error: "Missing required environment variable"
Check your `.env` file has:
```
OPENROUTER_API_KEY=sk-or-...
PLANT_ID_API_KEY=...
```

### Error: "Image file not found"
Verify the image path exists:
```bash
ls -la src/plant_images/
```

### Diagnosis stuck or not responding
- Check your internet connection
- Verify OpenRouter API key is valid
- Check API credit balance

---

## 📝 Understanding the Diagnostic Kernel

### What Makes It Special?

The diagnostic kernel is an **AI-driven decision engine** that:

1. **Dynamically generates code** - The AI writes Python code on-the-fly to determine what to ask next
2. **Executes safely** - All AI-generated code runs in a sandboxed Python environment
3. **Maintains context** - Tracks conversation history, hypotheses, and plant vitals
4. **Self-directs** - The AI decides when it has enough information to make a diagnosis

### Architecture Flow

```
User Problem → AI (OpenRouter) → Python Code → Sandbox Execution → Action Decision
                     ↑                                                      ↓
                     └──────────── User Response ←──────────── ASK_USER ←──┘
```

### Example of AI-Generated Code

When you describe a problem, the AI might generate code like:

```python
def next_action(context):
    if "watering" not in context.get("answers", []):
        return ASK_USER("How often do you water your plant?")
    
    if context["soil_moisture"] == "always wet":
        return CONCLUDE(
            finding="Overwatering",
            recommendation="Reduce watering frequency..."
        )
    
    return ASK_USER("Is the soil well-draining?")
```

This code is executed safely and determines the next step in the diagnosis.

---

## 🎯 Quick Reference Commands

| Command | Description | Example |
|---------|-------------|---------|
| `add` | Add a new plant | `cargo run -- add --image photo.jpg` |
| `list` | List all plants | `cargo run -- list` |
| `show` | Show plant details | `cargo run -- show <PLANT_ID>` |
| `delete` | Delete a plant | `cargo run -- delete <PLANT_ID>` |
| `diagnose` | Start diagnosis | `cargo run -- diagnose <PLANT_ID> --problem "issue"` |
| `history` | View diagnosis history | `cargo run -- history <PLANT_ID>` |
| `care` | Generate care schedule | `cargo run -- care "Plant Name"` |

---

## 📞 Support

If you encounter issues:
1. Check the troubleshooting section above
2. Verify your API keys are valid and have credits
3. Check the `data/` directory permissions
4. Review the error messages carefully

Happy plant diagnosing! 🌱

