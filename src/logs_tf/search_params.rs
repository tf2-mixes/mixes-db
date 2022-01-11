use std::cmp;

use reqwest::blocking::RequestBuilder;

use crate::SteamID;

pub struct SearchParams
{
    pub player_id: Option<SteamID>,
    pub title:     Option<String>,
    pub limit:     Option<u16>,
}

impl SearchParams
{
    pub fn player_id(id: SteamID) -> Self
    {
        Self {
            player_id: Some(id),
            title:     None,
            limit:     None,
        }
    }

    pub fn log_title(title: String) -> Self
    {
        Self {
            player_id: None,
            title:     Some(title),
            limit:     None,
        }
    }

    pub fn limit(limit: u16) -> Self
    {
        let limit = cmp::min(limit, 10000);

        Self {
            player_id: None,
            title:     None,
            limit:     Some(limit),
        }
    }

    pub fn add_player_id(mut self, id: SteamID) -> Self
    {
        self.player_id.replace(id);
        self
    }

    pub fn add_log_title(mut self, title: String) -> Self
    {
        self.title.replace(title);
        self
    }

    pub fn add_limit(mut self, limit: u16) -> Self
    {
        self.limit.replace(cmp::min(limit, 10000));
        self
    }

    pub fn add_params_to_request(self, request_builder: RequestBuilder) -> RequestBuilder
    {
        let request_builder = match self.player_id {
            Some(id) => request_builder.query(&[("player", &id.to_id64_string())]),
            None => request_builder,
        };

        let request_builder = match self.title {
            Some(name) => request_builder.query(&[("title", &name)]),
            None => request_builder,
        };

        let request_builder = match self.limit {
            Some(limit) => request_builder.query(&[("limit", &limit.to_string())]),
            None => request_builder,
        };

        request_builder
    }
}
