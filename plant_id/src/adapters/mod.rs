/*!
 * ADAPTERS
 *
 * Secondary adapters that interact with external services.
 * These implement the hexagonal architecture's secondary ports.
 */

pub mod ai_adapter;
pub mod plant_id_adapter;
pub mod storage_adapter;
pub mod sandbox_executor;

pub use ai_adapter::AiAdapter;
pub use plant_id_adapter::PlantIdAdapter;
pub use storage_adapter::StorageAdapter;
pub use sandbox_executor::{SandboxExecutor, ActionEffect};

