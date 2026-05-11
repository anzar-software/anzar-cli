use mongodb::{Database, IndexModel, bson::doc, options::IndexOptions};

use crate::domain::model::{PasswordResetToken, RolePermission, User, UserRole};

pub struct MongodbIndexes {
    pub db: Database,
}
impl MongodbIndexes {
    pub async fn create_unique_email_index(&self) -> Result<(), mongodb::error::Error> {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(options)
            .build();

        self.db
            .collection::<User>("users")
            .create_index(model)
            .await?;

        Ok(())
    }

    pub async fn create_token_hash_index(&self) -> Result<(), mongodb::error::Error> {
        let options = IndexOptions::builder()
            .unique(true)
            // TODO: implement TTL index, for auto removing
            // use std::time::Duration;
            // .expire_after(Some(Duration::from_secs(60 * 60))) // 30 minutes
            .build();
        let model = IndexModel::builder()
            .keys(doc! { "token": 1 })
            .options(options)
            .build();

        self.db
            .collection::<PasswordResetToken>("password_reset_tokens")
            .create_index(model)
            .await?;

        Ok(())
    }

    pub async fn create_role_permission_index(&self) -> Result<(), mongodb::error::Error> {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! { "roleId": 1, "permissionId": 1 })
            .options(options)
            .build();

        self.db
            .collection::<RolePermission>("role_permissions")
            .create_index(model)
            .await?;

        Ok(())
    }
    pub async fn create_user_role_index(&self) -> Result<(), mongodb::error::Error> {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! { "roleId": 1, "userId": 1 })
            .options(options)
            .build();

        self.db
            .collection::<UserRole>("user_roles")
            .create_index(model)
            .await?;

        Ok(())
    }
}
