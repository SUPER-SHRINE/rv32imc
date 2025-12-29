/// RISC-V の特権モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeMode {
    User       = 0,
    Supervisor = 1,
    Machine    = 3,
}
