use diesel::Queryable;

use rouille::router;

use serde::Deserialize;
use serde::Serialize;

use url::form_urlencoded;

use log::{trace, warn};

use crate::errors::{WebdevError, WebdevErrorKind};

use crate::search::{NullableSearch, Search};

use crate::users::models::UserList;
use super::schema::{access, user_access};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Access {
    pub id: i64,
    pub access_name: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "access"]
pub struct NewAccess {
    pub access_name: String,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name = "access"]
pub struct PartialAccess {
    pub access_name: String,
}

pub enum AccessRequest {
    GetAccess(i64), //id of access name searched
    CreateAccess(NewAccess), //new access type of some name to be created
    UpdateAccess(i64, PartialAccess), //Contains id to be changed to new access_name
    DeleteAccess(i64), //if of access to be deleted
}

impl AccessRequest {
    pub fn from_rouille(request: &rouille::Request) -> Result<AccessRequest, WebdevError> {
        trace!("Creating AccessRequest from {:#?}", request);

        router!(request,
            (GET) (/{id: i64}) => {
                Ok(AccessRequest::GetAccess(id))
            },

            (POST) (/) => {
                let request_body = request.data().ok_or(WebdevError::new(WebdevErrorKind::Format))?;
                let new_access: NewAccess = serde_json::from_reader(request_body)?;

                Ok(AccessRequest::CreateAccess(new_access))
            },

            (POST) (/{id: i64}) => {
                let request_body = request.data().ok_or(WebdevError::new(WebdevErrorKind::Format))?;
                let update_access: PartialAccess = serde_json::from_reader(request_body)?;

                Ok(AccessRequest::UpdateAccess(id, update_access))
            },

            (DELETE) (/{id: i64}) => {
                Ok(AccessRequest::DeleteAccess(id))
            },

            _ => {
                warn!("Could not create an access request for the given rouille request");
                Err(WebdevError::new(WebdevErrorKind::NotFound))
            }
        ) //end router

    }
}

pub enum AccessResponse {
    OneAccess(Access),
    NoResponse,
}

impl AccessResponse {
    pub fn to_rouille(self) -> rouille::Response {
        match self {
            AccessResponse::OneAccess(access) => rouille::Response::json(&access),
            AccessResponse::NoResponse => rouille::Response::empty_204(),
        }
    }
}



#[derive(Queryable, Serialize, Deserialize)]
pub struct UserAccess {
    pub permission_id: i64,
    pub access_id: i64,
    pub user_id: i64,
    pub permission_level: Option<String>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "user_access"]
pub struct NewUserAccess {
    pub access_id: i64,
    pub user_id: i64,
    pub permission_level: Option<String>,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name = "user_access"]
pub struct PartialUserAccess {
    pub access_id: i64,
    pub user_id: i64,
    pub permission_level: Option<Option<String>>,
}

pub struct SearchUserAccess {
    pub access_id: Search<i64>,
    pub user_id: Search<i64>,
    pub permission_level: NullableSearch<String>,
}

pub enum UserAccessRequest {
    SearchAccess(SearchUserAccess), //list of users with access id or (?) name
    CheckAccess(i64, i64), //entry allowing user of user_id to perform action of action_id
    CreateAccess(NewUserAccess), //entry to add to database
    UpdateAccess(i64, PartialUserAccess), //entry to update with new information
    DeleteAccess(i64), //entry to delete from database
}

impl UserAccessRequest {
    pub fn from_rouille(request: &rouille::Request) -> Result<UserAccessRequest, WebdevError> {
        trace!("Creating UserAccessRequest from {:#?}", request);

        let url_queries = form_urlencoded::parse(request.raw_query_string().as_bytes());

        router!(request,
            (GET) (/) => {

                let mut access_id = Search::NoSearch;
                let mut user_id = Search::NoSearch;
                let mut permission_level = NullableSearch::NoSearch;

                for (field, query) in url_queries {
                    match field.as_ref() {
                        "access_id" => access_id = Search::from_query(query.as_ref())?,
                        "user_id" => user_id = Search::from_query(query.as_ref())?,
                        "permission_level" => permission_level = NullableSearch::from_query(query.as_ref())?,
                        _ => return Err(WebdevError::new(WebdevErrorKind::Format)),
                    }
                }

                Ok(UserAccessRequest::SearchAccess(SearchUserAccess {
                    access_id,
                    user_id,
                    permission_level,
                }))
            },

            (GET) (/{user_id:i64}/{access_id: i64}) => {
                Ok(UserAccessRequest::CheckAccess(user_id, access_id))
            },

            (POST) (/) => {
                let request_body = request.data().ok_or(WebdevError::new(WebdevErrorKind::Format))?;
                let new_user_access: NewUserAccess = serde_json::from_reader(request_body)?;

                Ok(UserAccessRequest::CreateAccess(new_user_access))
            },

            (POST) (/{id: i64}) => {
                let request_body = request.data().ok_or(WebdevError::new(WebdevErrorKind::Format))?;
                let update_user_access: PartialUserAccess = serde_json::from_reader(request_body)?;

                Ok(UserAccessRequest::UpdateAccess(id, update_user_access))
            },

            (DELETE) (/{id: i64}) => {
                Ok(UserAccessRequest::DeleteAccess(id))
            },

            _ => {
                warn!("Could not create a user access request for the given rouille request");
                Err(WebdevError::new(WebdevErrorKind::NotFound))
            }
        ) //end router
    }
}

pub enum UserAccessResponse {
    ManyUsers(UserList),
    AccessState(bool),
    OneUserAccess(UserAccess),
    NoResponse,
}

impl UserAccessResponse {
    pub fn to_rouille(self) -> rouille::Response {
        match self {
            UserAccessResponse::ManyUsers(users) => rouille::Response::json(&users),
            UserAccessResponse::AccessState(state) => rouille::Response::text(if state {"true"} else {"false"}),
            UserAccessResponse::OneUserAccess(user_access) => rouille::Response::json(&user_access),
            UserAccessResponse::NoResponse => rouille::Response::empty_204(),
        }
    }
}
