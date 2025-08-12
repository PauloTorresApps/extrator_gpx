// js/map-manager.js - Gerenciamento do mapa e pontos de sincronização

// Variáveis de estado do mapa
let map = null;
let trackLayer = null;
let userMarker = null;
let suggestionMarker = null;
let gpxDataPoints = [];
let selectedSyncPoint = null;

// Inicialização do mapa
map = L.map('map').setView([0, 0], 2);
L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', { 
    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors' 
}).addTo(map);

// Ícones personalizados
const suggestionIcon = L.icon({ 
    iconUrl: 'data:image/svg+xml;base64,' + btoa(`<svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41"><path fill="#2E8B57" stroke="#FFFFFF" stroke-width="1.5" d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/><circle fill="#FFFFFF" cx="12.5" cy="12.5" r="4"/></svg>`), 
    iconSize: [25, 41], 
    iconAnchor: [12, 41], 
    popupAnchor: [1, -34] 
});

const userIcon = L.icon({ 
    iconUrl: 'data:image/svg+xml;base64=' + btoa(`<svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41"><path fill="#03dac6" stroke="#121212" stroke-width="1" d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/><circle fill="#121212" cx="12.5" cy="12.5" r="4"/></svg>`), 
    iconSize: [25, 41], 
    iconAnchor: [12, 41], 
    popupAnchor: [1, -34] 
});

function displayTrack(points) { 
    if (trackLayer) map.removeLayer(trackLayer); 
    const latLngs = points.map(p => [p.lat, p.lon]); 
    trackLayer = L.polyline(latLngs, { color: '#bb86fc', weight: 3, opacity: 0.8 }).addTo(map); 
    const bounds = trackLayer.getBounds(); 
    if (bounds.isValid()) { map.fitBounds(bounds.pad(0.1)); } 
    trackLayer.on('click', (e) => { 
        let closestPoint = null, minDistance = Infinity; 
        gpxDataPoints.forEach(p => { 
            const distance = map.distance([p.lat, p.lon], e.latlng); 
            if (distance < minDistance) { 
                minDistance = distance; 
                closestPoint = p; 
            } 
        }); 
        if (closestPoint) { 
            selectSyncPoint(closestPoint, false); 
        } 
    }); 
}

function selectSyncPoint(point, isSuggestion) {
    selectedSyncPoint = point; 
    if (userMarker) map.removeLayer(userMarker); 
    if (suggestionMarker) map.removeLayer(suggestionMarker); 
    
    const iconToUse = isSuggestion ? suggestionIcon : userIcon; 
    const newMarker = L.marker([point.lat, point.lon], { icon: iconToUse }).addTo(map); 
    
    if(isSuggestion) { suggestionMarker = newMarker; } else { userMarker = newMarker; } 
    
    const pointTime = point.displayTime || new Date(point.time).toLocaleString(currentLang.startsWith('en') ? 'en-US' : 'pt-BR', { timeZone: 'UTC' });
    syncPointInfo.textContent = t('sync_point_selected', { 
        type: isSuggestion ? t('suggestion_type') : t('manual_type'), 
        time: pointTime 
    }); 
    
    if (!isSuggestion) {
        notify.success(t('notification_sync_selected'), pointTime);
    }
    
    if (videoFile) { positionSection.style.display = 'block'; }
    validateGenerateButton();
}