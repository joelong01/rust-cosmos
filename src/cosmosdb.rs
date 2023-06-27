use crate::log_return_err;
/**
 *  this is the class that calls directly to CosmosDb
 */
use crate::models::{CosmosSecrets, User};
use crate::utility::{get_id, COLLECTION_NAME, DATABASE_NAME};
use anyhow::Result;
use azure_core::error::ErrorKind;
use log::error;

use azure_data_cosmos::prelude::{
    AuthorizationToken, CollectionClient, CosmosClient, DatabaseClient, Query,
};
use futures::StreamExt;
use log::info;
/**
 *  this is a convinient way to pass around meta data about CosmosDb.  UserDb will also expose methods for calling
 *  cosmos (see below)
 */
pub struct UserDb {
    client: Option<CosmosClient>,
    database: Option<DatabaseClient>,
    users_collection: Option<CollectionClient>,
}
/**
 *  We only use the public client in this sample.
 *
 *  there are other sample out there that do ::from_resource() for the auth token.  To set this token, do to the
 *  Azure portal and pick your CosmosDb, then pick your "Keys" on the left pane.  You'll see a page that shows
 *  secrets -- "PRIMARY KEY", "SECONDARY KEY", etc.  Click on the eye for the SECONDARY KEY so you see the content
 *  in clear text, and then copy it when the devsecrets.sh script asks for the Cosmos token.  That key needs to
 *  be converted to base64 using primary_from_base64()
 */
fn public_client(account: &str, token: &str) -> CosmosClient {
    let auth_token = match AuthorizationToken::primary_from_base64(token) {
        Ok(token) => token,
        Err(e) => panic!("Failed to create authorization token: {}", e),
    };

    CosmosClient::new(account, auth_token)
}

/**
 *  load the secrets from environment variables and store them in a CosmosSecrets struct
 */
pub fn get_cosmos_secrets() -> Result<CosmosSecrets, Box<dyn std::error::Error>> {
    let token = std::env::var("COSMOS_AUTH_TOKEN")
        .map_err(|_| "Set env variable COSMOS_AUTH_TOKEN first!")?;

    let account = std::env::var("COSMOS_ACCOUNT_NAME")
        .map_err(|_| "Set env variable COSMOS_ACCOUNT_NAME first!")?;

    Ok(CosmosSecrets { token, account })
}

/**
 *  this is the scruct that contains methods to manipulate cosmosdb.  the idea is to be able to write code like
 *
 *      let mut user_db = UserDb::new().await;
 *      user_db.connec();
 *      user_db.list();
 *      user_db.create(...)
 */

impl UserDb {
    pub async fn new() -> Self {
        match get_cosmos_secrets() {
            Ok(secrets) => {
                let client = public_client(secrets.account.as_str(), secrets.token.as_str());
                let database = client.database_client(DATABASE_NAME);
                let collection = database.collection_client(COLLECTION_NAME);
                Self {
                    // I have my token and my account name
                    client: Some(client),
                    database: Some(database),
                    users_collection: Some(collection),
                }
            }
            Err(..) => {
                error!("{}",
                "Error getting cosmos secrets.  if they have been set with devsecrets.sh, you need to restart vscode!");
                Self {
                    client: None,
                    database: None,
                    users_collection: None,
                }
            }
        }
    }

    /**
     *  setup the database to make the sample work.  NOTE:  this will DELETE the database first.  to call this:
     *
     *  let userdb = UserDb::new();
     *  userdb.setupdb()
     */
    pub async fn setupdb(&self) -> azure_core::error::Result<()> {
        info!("Deleting existing database");

        match self.database.as_ref().unwrap().delete_database().await {
            Ok(..) => info!("\tDeleted {} database", DATABASE_NAME),
            Err(e) => {
                if format!("{}", e).contains("404") {
                    info!("\tDatabase {} not found", DATABASE_NAME);
                } else {
                    log_return_err!(e)
                }
            }
        }

        info!("Creating new database");
        match self
            .client
            .as_ref()
            .unwrap()
            .create_database(DATABASE_NAME)
            .await
        {
            Ok(..) => info!("\tCreated database"),
            Err(e) => log_return_err!(e),
        }

        info!("Creating collections");
        match self
            .database
            .as_ref()
            .unwrap()
            // note: this is where the field for the partion key is set -- if you change anything, make sure this is
            // a member of your document struct!
            .create_collection(COLLECTION_NAME, "/partition_key")
            .await
        {
            Ok(..) => info!("\tCreated {} collection", COLLECTION_NAME),
            Err(e) => log_return_err!(e),
        }

        // add some users to the collection
        let user = User {
            email: "test@outlook.com".to_string(),
            partition_key: 1,
            name: "joe".to_string(),
            id: format!("unique_id{}", get_id()),
        };
        match self
            .database
            .as_ref()
            .unwrap()
            .collection_client(COLLECTION_NAME)
            .create_document(user)
            .await
        {
            Ok(..) => info!("\tCreated root document"),
            Err(e) => log_return_err!(e),
        }

        Ok(())
    }
    /**
     *  this will return *all* (non paginated) Users in the collection
     */
    pub async fn list(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        if let Some(collection) = &self.users_collection {
            let mut stream = collection
                .list_documents()
                .into_stream::<serde_json::Value>();
            let mut all_users = Vec::<User>::new();

            while let Some(response) = stream.next().await {
                match response {
                    Ok(response) => {
                        info!("\n{:#?}", response);
                        for document in response.documents {
                            // Process the document
                            let user: User = serde_json::from_value(document.document)?;
                            all_users.push(user);
                        }
                    }
                    Err(e) => {
                        log::error!("{}", e);
                        break;
                    }
                }
            }

            Ok(all_users)
        } else {
            log::error!("Collection is empty");
            return Err("Collection is empty".into());
        }
    }
    /**
     *  an api that creates a user in the cosmosdb users collection. in this sample, we return
     *  the full User object in the body, giving the client the partition_key and user id
     */
    pub async fn create_user(&self, user: User) -> azure_core::error::Result<()> {
        match self
            .users_collection
            .as_ref()
            .unwrap()
            .create_document(user.clone())
            .await
        {
            Ok(..) => match serde_json::to_string(&user.clone()) {
                Ok(..) => Ok(()),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e),
        }
    }

    /**
     *  an api that finds a user by the id in the cosmosdb users collection.
     */
    pub async fn find_user(&self, userid: &str) -> Result<User, azure_core::Error> {
        let query = format!(
            r#"SELECT * FROM {} u WHERE u.id = '{}'"#,
            COLLECTION_NAME, userid
        );

        let query = Query::new(query);

        let mut stream = self
            .users_collection
            .as_ref()
            .unwrap()
            .query_documents(query)
            .into_stream::<serde_json::Value>();
        //
        // this just matches what list does, but only returns the first one
        // we are getting an error right now, but nothing to indicate what the error is.
        while let Some(response) = stream.next().await {
            match response {
                Ok(response) => {
                    info!("\n{:#?}", response);
                    for doc in response.documents() {
                        // Process the document
                        let user: User = serde_json::from_value(doc.clone())?;
                        return Ok(user); // return user if found
                    }
                }
                Err(e) => {
                    log_return_err!(e)
                }
            }
        }
        Err(azure_core::Error::new(ErrorKind::Other, "User not found")) // return error if user not found
    }
}
