let map = null;
let trackLayer = null;
let userMarker = null;
let suggestionMarker = null;

const suggestionIcon = L.icon({ 
    iconUrl: 'data:image/svg+xml;base64,' + btoa(`<svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41"><path fill="#2E8B57" stroke="#FFFFFF" stroke-width="1.5" d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/><circle fill="#FFFFFF" cx="12.5" cy="12.5" r="4"/></svg>`), 
    iconSize: [25, 41], iconAnchor: [12, 41], popupAnchor: [1, -34] 
});

const userIcon = L.icon({ 
    iconUrl: 'data:image/svg+xml;base64,' + btoa(`<svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41"><path fill="#03dac6" stroke="#121212" stroke-width="1" d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/><circle fill="#121212" cx="12.5" cy="12.5" r="4"/></svg>`), 
    iconSize: [25, 41], iconAnchor: [12, 41], popupAnchor: [1, -34] 
});

export function initializeMap(mapId) {
    map = L.map(mapId).setView([0, 0], 2);
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', { 
        attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors' 
    }).addTo(map);
    return map;
}

export function displayTrack(points, onPointClickCallback) {
    if (trackLayer) map.removeLayer(trackLayer);
    
    const latLngs = points.map(p => [p.lat, p.lon]);
    trackLayer = L.polyline(latLngs, { color: '#bb86fc', weight: 3, opacity: 0.8 }).addTo(map);
    
    const bounds = trackLayer.getBounds();
    if (bounds.isValid()) map.fitBounds(bounds.pad(0.1));
    
    trackLayer.on('click', (e) => {
        let closestPoint = null, minDistance = Infinity;
        points.forEach(p => {
            const distance = map.distance([p.lat, p.lon], e.latlng);
            if (distance < minDistance) {
                minDistance = distance;
                closestPoint = p;
            }
        });
        if (closestPoint) onPointClickCallback(closestPoint, false);
    });
}

export function placeSyncMarker(point, isSuggestion) {
    if (isSuggestion) {
        if (suggestionMarker) map.removeLayer(suggestionMarker);
        suggestionMarker = L.marker([point.lat, point.lon], { icon: suggestionIcon }).addTo(map);
    } else {
        if (userMarker) map.removeLayer(userMarker);
        userMarker = L.marker([point.lat, point.lon], { icon: userIcon }).addTo(map);
    }
}

export function invalidateMapSize() {
    if (map) {
        setTimeout(() => map.invalidateSize(), 100);
    }
}