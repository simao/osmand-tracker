import * as _ from "https://unpkg.com/leaflet@1.7.1/dist/leaflet.css"
import L from 'https://esm.sh/leaflet';

// import * as L from 'leaflet';

// import * as i from 'leaflet/dist/images';


// import { ulid } from 'ulid';
// import * as dot from 'dot';

async function init() {
    // https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png

    let userId = "01FVFR45DD0Y2DG9TNFV8X5576";

    let httpResp = await fetch('/tracking?user_id=' + userId);

    type TrackingPoint = {
        user: string,
        lat: number,
        lon: number,
        speed: number,
        hdop: number | null,
        timestamp: String
    }

    type Response = {
        values: Array<TrackingPoint>,
    }

    let resp: Response = await httpResp.json();

    // TODO: values is empty

    let point = resp.values[0];

    var map = L.map('map').setView([point.lat, point.lon], 13);

    L.control.scale().addTo(map);

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: 'Map data © <a href="https://openstreetmap.org">OpenStreetMap</a> contributors',
        maxZoom: 18,
        tileSize: 512,
        zoomOffset: -1
    }).addTo(map);

    for (let i = 0; i < resp.values.length; i++) {
        let point = resp.values[i];
        var marker = L.marker([point.lat, point.lon]).addTo(map);
    }
}


init()
