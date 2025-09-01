

pub const SOURCE_HOST: &str = "hianime.to";
pub const SOURCE_REFERER: &str = "https://hianime.to/";
pub const SERVER_HOST: &str = "megacloud.blog";
pub const SERVER_REFERER: &str = "https://megacloud.blog/";


pub mod search;
pub mod get_episode_list;
pub mod get_episode_server;
pub mod get_server;
pub mod free_ptr;

#[cfg(test)]
mod test;