use crate::workspace::models::Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Rotate,
    Approve,
    View,
    Audit,
    Policy,
    Workspace,
}

impl Role {
    pub fn has_permission(&self, permission: Permission) -> bool {
        match self {
            Role::Owner => true,
            Role::Admin => !matches!(permission, Permission::Workspace),
            Role::Operator => matches!(permission, Permission::Rotate | Permission::View),
            Role::Viewer => matches!(permission, Permission::View),
            Role::Auditor => matches!(permission, Permission::Audit | Permission::View),
        }
    }

    pub fn can_manage_members(&self) -> bool {
        matches!(self, Role::Owner | Role::Admin)
    }

    pub fn can_manage_workspace(&self) -> bool {
        matches!(self, Role::Owner)
    }
}
