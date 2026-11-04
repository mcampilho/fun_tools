//! Public IP, location and weather, with short timeouts, provider fallbacks
//! and a small on-disk cache so repeated runs need no network at all.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CACHE_TTL_SECS: u64 = 15 * 60;

/// Tried in order; the first answer that parses wins.
const GEO_PROVIDERS: &[&str] = &[
    "https://ipwho.is/",
    "https://ipapi.co/json/",
    "http://ip-api.com/json/",
];

#[derive(Clone, Serialize, Deserialize)]
pub struct Geo {
    pub ip: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Weather {
    pub temp_c: f64,
    pub code: u32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct NetInfo {
    pub geo: Option<Geo>,
    pub weather: Option<Weather>,
}

#[derive(Serialize, Deserialize)]
struct CacheFile {
    saved_at: u64,
    info: NetInfo,
}

pub fn collect(refresh: bool) -> NetInfo {
    if !refresh {
        if let Some(info) = read_cache() {
            return info;
        }
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_millis(1200))
        .timeout(Duration::from_millis(2000))
        .user_agent(concat!("rfetch/", env!("CARGO_PKG_VERSION")))
        .build();

    let geo = GEO_PROVIDERS.iter().find_map(|url| {
        let v: Value = agent.get(url).call().ok()?.into_json().ok()?;
        parse_geo(&v)
    });
    let weather = geo
        .as_ref()
        .and_then(|g| Some((g.lat?, g.lon?)))
        .and_then(|(lat, lon)| fetch_weather(&agent, lat, lon));

    let info = NetInfo { geo, weather };
    if info.geo.is_some() {
        write_cache(&info);
    }
    info
}

fn parse_geo(v: &Value) -> Option<Geo> {
    let failed = v.get("success").and_then(Value::as_bool) == Some(false)
        || v.get("error").and_then(Value::as_bool) == Some(true)
        || v.get("status").and_then(Value::as_str) == Some("fail");
    if failed {
        return None;
    }
    let s = |keys: &[&str]| {
        keys.iter().find_map(|k| {
            v.get(*k)?.as_str().map(str::trim).filter(|s| !s.is_empty()).map(String::from)
        })
    };
    let f = |keys: &[&str]| keys.iter().find_map(|k| v.get(*k)?.as_f64());
    Some(Geo {
        ip: s(&["ip", "query"])?,
        // ipapi.co uses "country" for the code, so prefer "country_name".
        country: s(&["country_name", "country"]).filter(|c| c.len() > 2),
        country_code: s(&["country_code", "countryCode"]),
        city: s(&["city"]),
        lat: f(&["latitude", "lat"]),
        lon: f(&["longitude", "lon"]),
    })
}

fn fetch_weather(agent: &ureq::Agent, lat: f64, lon: f64) -> Option<Weather> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat:.4}&longitude={lon:.4}&current=temperature_2m,weather_code"
    );
    let v: Value = agent.get(&url).call().ok()?.into_json().ok()?;
    let current = v.get("current")?;
    Some(Weather {
        temp_c: current.get("temperature_2m")?.as_f64()?,
        code: current.get("weather_code")?.as_u64()? as u32,
    })
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn cache_path() -> Option<PathBuf> {
    #[cfg(windows)]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(not(windows))]
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")));
    Some(base?.join("rfetch").join("net.json"))
}

fn read_cache() -> Option<NetInfo> {
    let text = std::fs::read_to_string(cache_path()?).ok()?;
    let cache: CacheFile = serde_json::from_str(&text).ok()?;
    (now().saturating_sub(cache.saved_at) < CACHE_TTL_SECS).then_some(cache.info)
}

/// Best effort: write to a temporary file, then rename, so readers never see half a file.
fn write_cache(info: &NetInfo) {
    let Some(path) = cache_path() else { return };
    let Some(dir) = path.parent() else { return };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    let cache = CacheFile { saved_at: now(), info: info.clone() };
    let Ok(json) = serde_json::to_string(&cache) else { return };
    let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
    if std::fs::write(&tmp, json).is_ok() && std::fs::rename(&tmp, &path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_providers() {
        let ipwho: Value = serde_json::from_str(
            r#"{"success":true,"ip":"1.2.3.4","country":"Portugal","country_code":"PT","city":"Lisbon","latitude":38.7,"longitude":-9.1}"#,
        ).unwrap();
        let g = parse_geo(&ipwho).unwrap();
        assert_eq!(g.country.as_deref(), Some("Portugal"));
        assert_eq!(g.lat, Some(38.7));

        let ipapi: Value = serde_json::from_str(
            r#"{"ip":"1.2.3.4","country":"PT","country_name":"Portugal","country_code":"PT","city":"Porto","latitude":41.1,"longitude":-8.6}"#,
        ).unwrap();
        assert_eq!(parse_geo(&ipapi).unwrap().country.as_deref(), Some("Portugal"));

        let ipapicom: Value = serde_json::from_str(
            r#"{"status":"success","query":"1.2.3.4","country":"Spain","countryCode":"ES","city":"Madrid","lat":40.4,"lon":-3.7}"#,
        ).unwrap();
        assert_eq!(parse_geo(&ipapicom).unwrap().country_code.as_deref(), Some("ES"));

        let fail: Value = serde_json::from_str(r#"{"success":false,"message":"limit"}"#).unwrap();
        assert!(parse_geo(&fail).is_none());
    }
}
