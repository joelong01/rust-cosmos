use crate::utility::get_id;
/**
 * this is the module where I define the structures needed for the data in Cosmos
 */
use azure_data_cosmos::CosmosEntity;
use serde::{Deserialize, Serialize};

/**
 *  Every CosmosDb document needs to define the partition_key.  In Rust we do this via this trait.
 */
impl CosmosEntity for User {
    type Entity = u64;

    fn partition_key(&self) -> Self::Entity {
        self.partition_key
    }
}

/**
 * this is the document stored in cosmosdb.  the "id" field and the "partition_key" field are "special" in that the
 * system needs them. if id is not specified, cosmosdb will create a guild for the id (and create an 'id' field), You
 * can partition on any value, but it should be something that works well with the partion scheme that cosmos uses.
 * for this sample, we assume the db size is small, so we just partion on a number that the sample always sets to 1
 *
 */

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: String,
    pub partition_key: u64,
    pub email: String,
    pub name: String,
}

/**
 *  we are exposing a Web api to use cosmos and so a client must pass in data to create a new document.  This sample does
 *  it via form data in a POST.  it could be anything -- parameters, pass in a JSON document in the body, etc.  I picked
 *  form data because it doesn't make the URL longer, it doesn't require sharing a structure with the client, and it
 *  scales as more profile information is added (simply add more name/value pairs to the form).   actix_web will deserialize
 *  the form data to a structure, which I called PartialUser because it contains the data that the client can create,
 *  in particular it does not have the partition_key or the id
 */
#[derive(Debug, Deserialize, Serialize)]
pub struct PartialUser {
    pub email: String,
    pub name: String,
}

/**
 *  this trait makes it easy to write code to convert from a PartialUser to a User
 */

impl From<PartialUser> for User {
    fn from(client_player: PartialUser) -> Self {
        // You will generate the player_id and number here
        let id = format!("unique_id{}", get_id());
        let partition_key = 1;

        User {
            id,
            partition_key,
            email: client_player.email,
            name: client_player.name,
        }
    }
}

/**
 *  the .devcontainer/required-secrets.json contains the list of secrets needed to run this application.  this stuctu
 *  holds them so that they are more convinient to use
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct CosmosSecrets {
    pub token: String,
    pub account: String,
}
