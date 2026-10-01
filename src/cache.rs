//! In-memory cache of fully rendered, pre-compressed responses.
//!
//! The hot path for a cached page is: hash lookup -> pick encoding -> return
//! bytes. No database, no template rendering, no compression per request.
//! Any write to the catalog calls [`PageCache::purge`], so pages are never
//! stale for longer than one request.

use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use dashmap::DashMap;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::sync::Arc;

/// Hard cap on distinct cached keys so a crawler walking bogus URLs cannot
/// grow the map without bound. Known routes number in the low thousands.
const MAX_ENTRIES: usize = 10_000;

pub struct Cached {
    pub identity: Bytes,
    pub gzip: Bytes,
    pub br: Bytes,
    pub etag: HeaderValue,
    pub content_type: HeaderValue,
    pub cache_control: HeaderValue,
}

#[derive(Default)]
pub struct PageCache {
    map: DashMap<String, Arc<Cached>>,
}

impl PageCache {
    pub fn get(&self, key: &str) -> Option<Arc<Cached>> {
        self.map.get(key).map(|e| e.value().clone())
    }

    pub fn insert(&self, key: String, body: Bytes, content_type: &str, cache_control: &str) -> Arc<Cached> {
        let entry = Arc::new(Cached::build(body, content_type, cache_control));
        if self.map.len() < MAX_ENTRIES {
            self.map.insert(key, entry.clone());
        }
        entry
    }

    pub fn purge(&self) {
        self.map.clear();
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Cached {
    pub fn build(body: Bytes, content_type: &str, cache_control: &str) -> Self {
        let etag = {
            let digest = Sha256::digest(&body);
            HeaderValue::from_str(&format!("\"{}\"", hex::encode(&digest[..16]))).unwrap()
        };
        let gzip = {
            let mut enc =
                flate2::write::GzEncoder::new(Vec::with_capacity(body.len() / 3), flate2::Compression::new(9));
            enc.write_all(&body).unwrap();
            Bytes::from(enc.finish().unwrap())
        };
        let br = {
            let mut out = Vec::with_capacity(body.len() / 3);
            {
                let mut enc = brotli::CompressorWriter::new(&mut out, 4096, 9, 22);
                enc.write_all(&body).unwrap();
            }
            Bytes::from(out)
        };
        Self {
            identity: body,
            gzip,
            br,
            etag,
            content_type: HeaderValue::from_str(content_type).unwrap(),
            cache_control: HeaderValue::from_str(cache_control).unwrap(),
        }
    }

    /// Build the response for a request, honouring `Accept-Encoding` and
    /// `If-None-Match`.
    pub fn respond(&self, req_headers: &HeaderMap) -> Response {
        if let Some(inm) = req_headers.get(header::IF_NONE_MATCH)
            && inm == self.etag
        {
            let mut resp = StatusCode::NOT_MODIFIED.into_response();
            resp.headers_mut().insert(header::ETAG, self.etag.clone());
            resp.headers_mut().insert(header::CACHE_CONTROL, self.cache_control.clone());
            return resp;
        }
        let accept = req_headers.get(header::ACCEPT_ENCODING).and_then(|v| v.to_str().ok()).unwrap_or("");
        let (body, encoding) = if accept.contains("br") {
            (self.br.clone(), Some("br"))
        } else if accept.contains("gzip") {
            (self.gzip.clone(), Some("gzip"))
        } else {
            (self.identity.clone(), None)
        };
        let mut resp = (StatusCode::OK, body).into_response();
        let h = resp.headers_mut();
        h.insert(header::CONTENT_TYPE, self.content_type.clone());
        h.insert(header::ETAG, self.etag.clone());
        h.insert(header::CACHE_CONTROL, self.cache_control.clone());
        h.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
        if let Some(enc) = encoding {
            h.insert(header::CONTENT_ENCODING, HeaderValue::from_static(enc));
        }
        resp
    }
}
