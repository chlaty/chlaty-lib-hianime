

pub const SOURCE_HOST: &str = "hianime.to";
pub const SOURCE_REFERER: &str = "https://hianime.to/";
pub const SERVER_HOST: &str = "megacloud.blog";
pub const SERVER_REFERER: &str = "https://megacloud.blog/";


mod search;
mod get_episode;
mod get_episode_list;
mod free_ptr;

#[cfg(test)]
mod test;