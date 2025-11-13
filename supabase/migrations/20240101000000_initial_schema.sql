-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create workspaces table
CREATE TABLE workspaces (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    plan_tier TEXT NOT NULL DEFAULT 'free' CHECK (plan_tier IN ('free', 'starter', 'pro', 'enterprise')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create workspace_members table
CREATE TABLE workspace_members (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('owner', 'admin', 'operator', 'viewer', 'auditor')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, user_id)
);

-- Create provider_configs table
CREATE TABLE provider_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    mode TEXT NOT NULL CHECK (mode IN ('hosted', 'oauth', 'kms', 'api_key')),
    config_jsonb JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, provider)
);

-- Create credentials table
CREATE TABLE credentials (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    secret_name TEXT NOT NULL,
    encrypted_value BYTEA NOT NULL,
    kms_key_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(workspace_id, provider, secret_name)
);

-- Create oauth_tokens table
CREATE TABLE oauth_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    encrypted_refresh_token BYTEA NOT NULL,
    access_token TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, provider)
);

-- Create api_keys table
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    key_hash TEXT NOT NULL UNIQUE,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

-- Create rotation_metering table
CREATE TABLE rotation_metering (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    rotation_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, date)
);

-- Create indexes for performance
CREATE INDEX idx_workspace_members_workspace_id ON workspace_members(workspace_id);
CREATE INDEX idx_workspace_members_user_id ON workspace_members(user_id);
CREATE INDEX idx_provider_configs_workspace_id ON provider_configs(workspace_id);
CREATE INDEX idx_credentials_workspace_id ON credentials(workspace_id);
CREATE INDEX idx_credentials_workspace_provider_secret ON credentials(workspace_id, provider, secret_name);
CREATE INDEX idx_oauth_tokens_workspace_id ON oauth_tokens(workspace_id);
CREATE INDEX idx_api_keys_workspace_id ON api_keys(workspace_id);
CREATE INDEX idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX idx_rotation_metering_workspace_id ON rotation_metering(workspace_id);
CREATE INDEX idx_rotation_metering_date ON rotation_metering(date);
CREATE INDEX idx_workspaces_created_at ON workspaces(created_at);
CREATE INDEX idx_workspace_members_created_at ON workspace_members(created_at);
CREATE INDEX idx_credentials_created_at ON credentials(created_at);

-- Enable Row Level Security on all tables
ALTER TABLE workspaces ENABLE ROW LEVEL SECURITY;
ALTER TABLE workspace_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE provider_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE rotation_metering ENABLE ROW LEVEL SECURITY;

-- RLS Policies for workspaces
CREATE POLICY "Users can view workspaces they are members of"
    ON workspaces FOR SELECT
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = workspaces.id
            AND workspace_members.user_id = auth.uid()
        )
    );

CREATE POLICY "Users can create their own workspaces"
    ON workspaces FOR INSERT
    WITH CHECK (true);

CREATE POLICY "Owners and admins can update workspaces"
    ON workspaces FOR UPDATE
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = workspaces.id
            AND workspace_members.user_id = auth.uid()
            AND workspace_members.role IN ('owner', 'admin')
        )
    );

CREATE POLICY "Only owners can delete workspaces"
    ON workspaces FOR DELETE
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = workspaces.id
            AND workspace_members.user_id = auth.uid()
            AND workspace_members.role = 'owner'
        )
    );

-- RLS Policies for workspace_members
CREATE POLICY "Users can view members of workspaces they belong to"
    ON workspace_members FOR SELECT
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members wm
            WHERE wm.workspace_id = workspace_members.workspace_id
            AND wm.user_id = auth.uid()
        )
    );

CREATE POLICY "Owners and admins can add members"
    ON workspace_members FOR INSERT
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = workspace_members.workspace_id
            AND workspace_members.user_id = auth.uid()
            AND workspace_members.role IN ('owner', 'admin')
        )
    );

CREATE POLICY "Owners and admins can update member roles"
    ON workspace_members FOR UPDATE
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members wm
            WHERE wm.workspace_id = workspace_members.workspace_id
            AND wm.user_id = auth.uid()
            AND wm.role IN ('owner', 'admin')
        )
    );

CREATE POLICY "Owners and admins can remove members"
    ON workspace_members FOR DELETE
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members wm
            WHERE wm.workspace_id = workspace_members.workspace_id
            AND wm.user_id = auth.uid()
            AND wm.role IN ('owner', 'admin')
        )
    );

-- RLS Policies for provider_configs
CREATE POLICY "Workspace members can view provider configs"
    ON provider_configs FOR SELECT
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = provider_configs.workspace_id
            AND workspace_members.user_id = auth.uid()
        )
    );

CREATE POLICY "Owners and admins can manage provider configs"
    ON provider_configs FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = provider_configs.workspace_id
            AND workspace_members.user_id = auth.uid()
            AND workspace_members.role IN ('owner', 'admin')
        )
    );

-- RLS Policies for credentials
CREATE POLICY "Users with rotate permission can view credentials"
    ON credentials FOR SELECT
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = credentials.workspace_id
            AND workspace_members.user_id = auth.uid()
            AND workspace_members.role IN ('owner', 'admin', 'operator')
        )
    );

CREATE POLICY "Users with rotate permission can manage credentials"
    ON credentials FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = credentials.workspace_id
            AND workspace_members.user_id = auth.uid()
            AND workspace_members.role IN ('owner', 'admin', 'operator')
        )
    );

-- RLS Policies for oauth_tokens
CREATE POLICY "Users with rotate permission can view oauth tokens"
    ON oauth_tokens FOR SELECT
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = oauth_tokens.workspace_id
            AND workspace_members.user_id = auth.uid()
            AND workspace_members.role IN ('owner', 'admin', 'operator')
        )
    );

CREATE POLICY "Users with rotate permission can manage oauth tokens"
    ON oauth_tokens FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = oauth_tokens.workspace_id
            AND workspace_members.user_id = auth.uid()
            AND workspace_members.role IN ('owner', 'admin', 'operator')
        )
    );

-- RLS Policies for api_keys
CREATE POLICY "Workspace members can view api keys"
    ON api_keys FOR SELECT
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = api_keys.workspace_id
            AND workspace_members.user_id = auth.uid()
        )
    );

CREATE POLICY "Owners and admins can manage api keys"
    ON api_keys FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = api_keys.workspace_id
            AND workspace_members.user_id = auth.uid()
            AND workspace_members.role IN ('owner', 'admin')
        )
    );

-- RLS Policies for rotation_metering
CREATE POLICY "Workspace members can view rotation metering"
    ON rotation_metering FOR SELECT
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = rotation_metering.workspace_id
            AND workspace_members.user_id = auth.uid()
        )
    );

CREATE POLICY "System can manage rotation metering"
    ON rotation_metering FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM workspace_members
            WHERE workspace_members.workspace_id = rotation_metering.workspace_id
            AND workspace_members.user_id = auth.uid()
            AND workspace_members.role IN ('owner', 'admin', 'operator')
        )
    );

-- Create function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add triggers for updated_at columns
CREATE TRIGGER update_workspaces_updated_at BEFORE UPDATE ON workspaces
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_provider_configs_updated_at BEFORE UPDATE ON provider_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_credentials_updated_at BEFORE UPDATE ON credentials
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_oauth_tokens_updated_at BEFORE UPDATE ON oauth_tokens
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create function to automatically add workspace creator as owner
CREATE OR REPLACE FUNCTION add_workspace_creator_as_owner()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO workspace_members (workspace_id, user_id, role)
    VALUES (NEW.id, auth.uid(), 'owner');
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add trigger to automatically add creator as owner
CREATE TRIGGER add_workspace_creator AFTER INSERT ON workspaces
    FOR EACH ROW EXECUTE FUNCTION add_workspace_creator_as_owner();

