use chrono::DateTime;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqlitePoolOptions;
use tide::Request;
use tide::prelude::*;
use ulid::Ulid;
use sqlx::prelude::*;
use std::fmt::Display;
use std::str::FromStr;
use chrono::prelude::*;
use tera::Tera;
use tide_tera::prelude::*;

#[derive(Clone, Debug)]
pub struct State {
    db: SqlitePool,
    tera: Tera
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TrackingGetParams {
    user_id: Ulid,
    later_than_epoch: Option<i64>,
    limit: Option<i64>,
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UserIdGetParams {
    user_id: Ulid,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct UserForm {
    username: String,
}

#[derive(Debug, Serialize)]
struct Response<T : serde::de::DeserializeOwned> {
    values: Vec<T>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct RecordKey(Ulid);

impl Display for RecordKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TrackingPoint {
    user: Ulid,
    #[serde(skip_serializing)]
    record_key: Option<RecordKey>,
    lat: f64,
    lon: f64,
    altitude: f64,
    bearing: String,
    speed: f64,
    hdop: Option<f64>,
    timestamp: i64,
    #[serde(skip_deserializing)]
    utc_timestamp: Option<DateTime<Utc>>
}

// Just generate a valid id, doesn't do anything else
async fn new_user(mut req: Request<State>) -> tide::Result {
    let user_id = Ulid::new();
    let record_key = RecordKey(Ulid::new());
    let user: UserForm = req.body_form().await?;

    // TODO: Pass should be hashed

    sqlx::query("insert into users (id, name, record_key, ts) values ($1, $2, $3, $4)")
        .bind(user_id.to_string())
        .bind(record_key.to_string())
        .bind(&user.username)
        .bind(chrono::Utc::now())
        .execute(&req.state().db).await?;

    // TODO: Should use POST => GET, but no simple way to pass password
    let tera = &req.state().tera;
    tera.render_response("show.html", &tide_tera::context! { "user_id" => user_id.to_string(), "name" => user.username, "pass" => record_key.0.to_string() })
}


async fn get_all(req: Request<State>) -> tide::Result {
    let params: TrackingGetParams = req.query()?;

    let later_than = Utc.timestamp_millis(params.later_than_epoch.unwrap_or(0));

    let query = "
       WITH
          ts_with_is_first(ts, is_first_ts) AS (
              SELECT ts, (julianday(ts) - julianday(lead(ts) OVER (order BY ts DESC))) * 24 > 5 AS is_first_ts FROM tracking_points
              WHERE user_id = $1
              ORDER BY ts desc
          ),
          first_ts_last_trip(ts) AS (
              SELECT tp.ts AS ts FROM tracking_points tp, ts_with_is_first WHERE tp.ts = ts_with_is_first.ts AND (ts_with_is_first.is_first_ts = 1 OR ts_with_is_first.is_first_ts is NULL) AND user_id = $1
              ORDER BY ts DESC LIMIT 1
       )
       SELECT user_id, lat, lon, altitude, speed, hdop, bearing, tp.ts speed FROM tracking_points tp, first_ts_last_trip
       WHERE
         tp.ts >= first_ts_last_trip.ts AND
         tp.ts > $2 AND
         tp.user_id = $1
       ORDER BY tp.ts DESC LIMIT $3
     ";

    let rows = sqlx::query(query)
        .bind(params.user_id.to_string())
        .bind(later_than)
        .bind(params.limit.unwrap_or(2000))
        .map(|r| {
            let id_str: String = r.get(0);
            let timestamp: DateTime<Utc> = r.get(7);
            TrackingPoint { user: Ulid::from_str(&id_str).unwrap(), record_key: None, lat: r.get(1), lon: r.get(2), altitude: r.get(3), speed: r.get(4), hdop: r.get(5), bearing: r.get(6), utc_timestamp: Some(timestamp), timestamp: timestamp.timestamp_millis() }
        })
        .fetch_all(&req.state().db)
        .await?;

    let response = Response { values: rows };

    let mut res = tide::Response::new(200);
    res.set_body(tide::Body::from_json(&response)?);
    Ok(res)
}


async fn show(req: Request<State>) -> tide::Result {
    let tera = &req.state().tera;
    let id = req.query::<UserIdGetParams>()?.user_id;

    // TODO: Index?
    let name: Option<String> = sqlx::query("select name from users where id = $1")
        .bind(id.to_string())
        .map(|r| r.get(0))
        .fetch_optional(&req.state().db).await?;

    tera.render_response("show.html", &tide_tera::context! { "user_id" => id.to_string(), "name" => name })
}


async fn record(req: Request<State>) -> tide::Result {
    let point: TrackingPoint = req.query()?;

    let date_time = Utc.timestamp_millis(point.timestamp);

    let now = Utc::now();

    // TODO: Verify record key, return 403

    sqlx::query("insert into tracking_points (user_id, lat, lon, altitude, speed, hdop, ts, received_at) values ($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind(point.user.to_string())
        .bind(point.lat)
        .bind(point.lon)
        .bind(point.altitude)
        .bind(point.speed)
        .bind(point.hdop)
        .bind(date_time)
        .bind(now)
        .execute(&req.state().db).await?;

    Ok(tide::Response::new(200))
}


async fn make_db_pool(url: &str) -> SqlitePool {
    let opts = SqliteConnectOptions::default()
        .filename(url)
        .foreign_keys(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    SqlitePoolOptions::default()
        .connect_with(opts).await.unwrap()
}


#[async_std::main]
async fn main() -> tide::Result<()> {
    let db_url = std::env::var("DATABASE_URL").unwrap_or("data/osmand-tracker.db".to_string());
    let dev_log = std::env::var("DEV_LOG").unwrap_or("1".to_string());

    let tera = Tera::new("dist/templates/**/*")?;

    let db_pool = make_db_pool(&db_url).await;
    let mut app = tide::with_state(State { db: db_pool, tera });

    if dev_log == "1" {
        tide::log::start();
        //   pretty_env_logger::init();
    } else {
        app.with(driftwood::ApacheCombinedLogger);
    }

    app.at("/").serve_file("./dist/index.html").unwrap();
    app.at("/").serve_dir("./dist").unwrap();

    app.at("/record").get(record);
    app.at("/show").get(show);
    app.at("/tracking").get(get_all);
    app.at("/users").post(new_user);

    app.listen("0.0.0.0:9000").await?;
    Ok(())
}
