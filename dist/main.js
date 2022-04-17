import L from 'https://esm.sh/leaflet@1.7.1'

async function fetchPoints(userId, laterThan) {
  var baseUrl = 'tracking?limit=2000&user_id=' + userId

  if (laterThan != "") {
    baseUrl += "&later_than_epoch=" + Math.floor(laterThan);
  }

  var httpResp = await fetch(baseUrl);
  var resp = await httpResp.json();

  return resp.values
}

function showPoints(settings, map, points) {
  if (points.length == 0) {
    console.log("No points received");
    return;
  }

  if (settings.lastCircle != undefined) {
    settings.lastCircle.setStyle({
      color: 'red',
      fillColor: '#f03',
      fillOpacity: 0.5
    })
  }

  var lastCircle = undefined;

  for (var i = points.length - 1; i >= 0; i--) {
    var point = points[i];

    var circle = L.circle([point.lat, point.lon], {
      color: 'red',
      fillColor: '#f03',
      fillOpacity: 0.5,
      radius: point.hdop
    }).addTo(map);

    var date = new Date(point.timestamp).toISOString();

    var speedKmh = point.speed * 3.6;

    circle.bindPopup("speed: " + speedKmh + ", time: " + date);

    lastCircle = circle;
  }

  var lastPoint = points[0];

  lastCircle.setStyle({
    color: "#006600",
    fillColor: "#006633",
    fillOpacity: 0.9,
  });

  // Only setView once
  if (settings.lastCircle == undefined) {
    map.setView([lastPoint.lat, lastPoint.lon], 15);
  }

  settings.lastCircle = lastCircle;
}

async function init() {
  var map = L.map('map').setView([38.763152, -9.169679], 13);

  L.control.scale().addTo(map);

  L.tileLayer('https://tile.thunderforest.com/cycle/{z}/{x}/{y}.png?apikey=35dd3431a2ae48f5878ccffda51f9de9', {
    attribution: 'Map data © <a href="https://openstreetmap.org">OpenStreetMap</a> contributors',
    maxZoom: 18
  }).addTo(map);

  return map
}

async function fetchMore(map, settings) {
  var resp = await fetchPoints(settings.userId, settings.laterThan);

  if (resp.length == 0) {
    return;
  }

  settings.laterThan = Number(new Date(resp[0].timestamp))

  showPoints(settings, map, resp)
}

document.addEventListener("DOMContentLoaded", async (_event) => {
  const params = new URLSearchParams(window.location.search);

  const userId = params.get("user_id");

  // "01FVFR45DD0Y2DG9TNFV8X5576",

  // TODO: Throw if userId is null

  var settings = {
    userId: userId,
    laterThan: 0,
    lastCircle: undefined
  };

  var map = await init();

  await fetchMore(map, settings);

  setInterval(fetchMore, 3000, map, settings);
});
