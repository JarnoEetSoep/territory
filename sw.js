var cacheName = "territory-1779302640";
var filesToCache = [
  "./",
  "./index.html",
  "./territory.js",
  "./territory_bg.wasm",
];

/* Start the service worker and cache all of the app"s content */
self.addEventListener("install", e => {
  e.waitUntil(
    caches.open(cacheName).then(cache => {
      return cache.addAll(filesToCache);
    })
  );
});

/* Serve cached content when offline */
self.addEventListener("fetch", e => {
  e.respondWith(
    caches.match(e.request).then(response => {
      return response || fetch(e.request);
    })
  );
});

self.addEventListener("activate", e => {
  e.waitUntil(
    caches.keys().then(async keys => {
      Promise.all(keys
        .filter(key => key != cacheName)
        .map(async key => await caches.delete(key))
      )
    })
  );
});
