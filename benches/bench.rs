#![feature(test)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate miniserde;

extern crate serde;
extern crate serde_json;

extern crate test;
use test::Bencher;

fn input_json() -> String {
    std::fs::read_to_string("benches/twitter.json").unwrap()
}

fn input_struct() -> Twitter {
    let j = input_json();
    serde_json::from_str(&j).unwrap()
}

#[bench]
fn bench_deserialize_miniserde(b: &mut Bencher) {
    let j = input_json();
    b.iter(|| {
        miniserde::json::from_str::<Twitter>(&j).unwrap();
    });
}

#[bench]
fn bench_deserialize_serdejson(b: &mut Bencher) {
    let j = input_json();
    b.iter(|| {
        serde_json::from_str::<Twitter>(&j).unwrap();
    });
}

#[bench]
fn bench_serialize_miniserde(b: &mut Bencher) {
    let s = input_struct();
    b.iter(|| {
        miniserde::json::to_string(&s);
    });
}

#[bench]
fn bench_serialize_serdejson(b: &mut Bencher) {
    let s = input_struct();
    b.iter(|| {
        serde_json::to_string(&s).unwrap();
    });
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct Twitter {
    statuses: Vec<Status>,
    search_metadata: SearchMetadata,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct Status {
    metadata: Metadata,
    created_at: String,
    id: u64,
    id_str: String,
    text: String,
    source: String,
    truncated: bool,
    in_reply_to_status_id: Option<u64>,
    in_reply_to_status_id_str: Option<String>,
    in_reply_to_user_id: Option<u32>,
    in_reply_to_user_id_str: Option<String>,
    in_reply_to_screen_name: Option<String>,
    user: User,
    geo: (),
    coordinates: (),
    place: (),
    contributors: (),
    retweeted_status: Option<Box<Status>>,
    retweet_count: u32,
    favorite_count: u32,
    entities: StatusEntities,
    favorited: bool,
    retweeted: bool,
    possibly_sensitive: Option<bool>,
    lang: String,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct Metadata {
    result_type: String,
    iso_language_code: String,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct User {
    id: u32,
    id_str: String,
    name: String,
    screen_name: String,
    location: String,
    description: String,
    url: Option<String>,
    entities: UserEntities,
    protected: bool,
    followers_count: u32,
    friends_count: u32,
    listed_count: u32,
    created_at: String,
    favourites_count: u32,
    utc_offset: Option<i32>,
    time_zone: Option<String>,
    geo_enabled: bool,
    verified: bool,
    statuses_count: u32,
    lang: String,
    contributors_enabled: bool,
    is_translator: bool,
    is_translation_enabled: bool,
    profile_background_color: String,
    profile_background_image_url: String,
    profile_background_image_url_https: String,
    profile_background_tile: bool,
    profile_image_url: String,
    profile_image_url_https: String,
    profile_banner_url: Option<String>,
    profile_link_color: String,
    profile_sidebar_border_color: String,
    profile_sidebar_fill_color: String,
    profile_text_color: String,
    profile_use_background_image: bool,
    default_profile: bool,
    default_profile_image: bool,
    following: bool,
    follow_request_sent: bool,
    notifications: bool,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct UserEntities {
    url: Option<UserUrl>,
    description: UserEntitiesDescription,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct UserUrl {
    urls: Vec<Url>,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct Url {
    url: String,
    expanded_url: String,
    display_url: String,
    indices: Indices,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct UserEntitiesDescription {
    urls: Vec<Url>,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct StatusEntities {
    hashtags: Vec<Hashtag>,
    symbols: Vec<()>,
    urls: Vec<Url>,
    user_mentions: Vec<UserMention>,
    media: Option<Vec<Media>>,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct Hashtag {
    text: String,
    indices: Indices,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct UserMention {
    screen_name: String,
    name: String,
    id: u32,
    id_str: String,
    indices: Indices,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct Media {
    id: u64,
    id_str: String,
    indices: Indices,
    media_url: String,
    media_url_https: String,
    url: String,
    display_url: String,
    expanded_url: String,
    #[serde(rename = "type")]
    media_type: String,
    sizes: Sizes,
    source_status_id: Option<u64>,
    source_status_id_str: Option<String>,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct Sizes {
    medium: Size,
    small: Size,
    thumb: Size,
    large: Size,
}

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct Size {
    w: u16,
    h: u16,
    resize: String,
}

type Indices = (u8, u8);

#[derive(Serialize, MiniSerialize, Deserialize, MiniDeserialize)]
struct SearchMetadata {
    completed_in: f32,
    max_id: u64,
    max_id_str: String,
    next_results: String,
    query: String,
    refresh_url: String,
    count: u8,
    since_id: u64,
    since_id_str: String,
}
