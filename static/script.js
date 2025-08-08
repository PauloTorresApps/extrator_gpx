// Elementos da UI
const gpxInput = document.getElementById('gpx-file');
const videoInput = document.getElementById('video-file');
const generateBtn = document.getElementById('generate-btn');
const statusDiv = document.getElementById('status');
const logsContainer = document.getElementById('logs-container');
const logsPre = document.getElementById('logs');
const downloadDiv = document.getElementById('download-link');
const downloadLink = downloadDiv.querySelector('a');
const syncPointInfo = document.getElementById('sync-point-info');
const gpxInfo = document.getElementById('gpx-info');
const videoInfo = document.getElementById('video-info');
const progressContainer = document.getElementById('progress-container');
const progressBar = document.getElementById('progress-bar');
const mapElement = document.getElementById('map');
const mapSection = document.getElementById('map-section');
const trackInfoDiv = document.getElementById('track-info');
const positionSection = document.getElementById('position-section');

// --- INÍCIO DA ALTERAÇÃO: Seletores para os novos controles ---
const speedoCheckbox = document.getElementById('add-speedo-overlay');
const trackCheckbox = document.getElementById('add-track-overlay');
const speedoPositionGrid = document.getElementById('speedo-position-grid');
const trackPositionGrid = document.getElementById('track-position-grid');
const speedoPositionRadios = document.querySelectorAll('input[name="speedoPosition"]');
const trackPositionRadios = document.querySelectorAll('input[name="trackPosition"]');
// --- FIM DA ALTERAÇÃO ---

// Variáveis de estado
let gpxFile = null, videoFile = null, selectedSyncPoint = null, map = null, trackLayer = null, userMarker = null, suggestionMarker = null, gpxDataPoints = [];

// Inicialização do mapa
map = L.map('map').setView([0, 0], 2);
L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', { attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors' }).addTo(map);

// Event Listeners
gpxInput.addEventListener('change', handleGpxUpload);
videoInput.addEventListener('change', handleVideoUpload);
generateBtn.addEventListener('click', handleGenerate);

// --- INÍCIO DA ALTERAÇÃO: Listeners para os novos controles ---
speedoCheckbox.addEventListener('change', () => {
    speedoPositionGrid.classList.toggle('hidden', !speedoCheckbox.checked);
    updatePositionControls();
});

trackCheckbox.addEventListener('change', () => {
    trackPositionGrid.classList.toggle('hidden', !trackCheckbox.checked);
    updatePositionControls();
});

speedoPositionRadios.forEach(radio => radio.addEventListener('change', updatePositionControls));
trackPositionRadios.forEach(radio => radio.addEventListener('change', updatePositionControls));
// --- FIM DA ALTERAÇÃO ---


function updatePositionControls() {
    const speedoPos = speedoCheckbox.checked ? document.querySelector('input[name="speedoPosition"]:checked').value : null;
    const trackPos = trackCheckbox.checked ? document.querySelector('input[name="trackPosition"]:checked').value : null;

    trackPositionRadios.forEach(radio => {
        radio.disabled = (radio.value === speedoPos);
    });

    speedoPositionRadios.forEach(radio => {
        radio.disabled = (radio.value === trackPos);
    });
}

async function fetchAndApplySuggestion() {
    if (!gpxFile || !videoFile) return;
    statusDiv.textContent = 'Analisando ficheiros para sugerir ponto e percurso...';
    const formData = new FormData();
    formData.append('gpxFile', gpxFile);
    formData.append('videoFile', videoFile);
    try {
        const response = await fetch('/suggest', { method: 'POST', body: formData });
        const data = await response.json();

        if (data.interpolated_points && data.interpolated_points.length > 0) {
            statusDiv.textContent = "Percurso de alta precisão carregado do servidor.";
            gpxDataPoints = data.interpolated_points.map(p => ({ ...p, time: new Date(p.time) }));
            displayTrack(gpxDataPoints);
        }

        if (response.ok && data.timestamp) {
            const suggestedPoint = { lat: data.latitude, lon: data.longitude, time: new Date(data.timestamp) };
            selectSyncPoint(suggestedPoint, true);
            statusDiv.textContent = 'Sugestão automática aplicada! Pode ajustar no mapa se necessário.';
        } else {
            if (!data.interpolated_points) {
                statusDiv.textContent = `Não foi possível obter sugestão: ${data.message || 'Erro desconhecido'}. Selecione um ponto manualmente.`;
            }
        }
    } catch (error) {
        console.error("Erro ao contactar o endpoint /suggest:", error);
        statusDiv.textContent = 'Erro de comunicação ao obter sugestão. Selecione um ponto manualmente.';
    }
}

const suggestionIcon = L.icon({ iconUrl: 'data:image/svg+xml;base64,' + btoa(`<svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41"><path fill="#2E8B57" stroke="#FFFFFF" stroke-width="1.5" d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/><circle fill="#FFFFFF" cx="12.5" cy="12.5" r="4"/></svg>`), iconSize: [25, 41], iconAnchor: [12, 41], popupAnchor: [1, -34] });
const userIcon = L.icon({ iconUrl: 'data:image/svg+xml;base64,' + btoa(`<svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41"><path fill="#03dac6" stroke="#121212" stroke-width="1" d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/><circle fill="#121212" cx="12.5" cy="12.5" r="4"/></svg>`), iconSize: [25, 41], iconAnchor: [12, 41], popupAnchor: [1, -34] });

function handleGpxUpload(event) {
    gpxFile = event.target.files[0];
    if (!gpxFile) return;
    gpxInfo.textContent = gpxFile.name;
    statusDiv.textContent = "Ficheiro GPX carregado. Selecione o ficheiro de vídeo.";
    videoInput.disabled = false;
    videoInfo.textContent = 'Agora pode selecionar o ficheiro de vídeo';
    fetchAndApplySuggestion();
}

function handleVideoUpload(event) {
    videoFile = event.target.files[0];
    if (!videoFile) return;
    videoInfo.textContent = videoFile.name;
    checkAndShowMapSection();
    fetchAndApplySuggestion();
}

function checkAndShowMapSection() { 
    if (gpxFile && videoFile) { 
        mapSection.style.display = 'block'; 
        syncPointInfo.style.display = 'block'; 
        setTimeout(() => map.invalidateSize(), 100); 
    } 
}

function displayTrack(points) { 
    if (trackLayer) map.removeLayer(trackLayer); 
    const latLngs = points.map(p => [p.lat, p.lon]); 
    trackLayer = L.polyline(latLngs, { color: '#bb86fc', weight: 3, opacity: 0.8 }).addTo(map); 
    const bounds = trackLayer.getBounds();
    if (bounds.isValid()) {
        map.fitBounds(bounds.pad(0.1));
    }
    trackLayer.on('click', (e) => { 
        const clickedLat = e.latlng.lat, clickedLon = e.latlng.lng; 
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
    if(isSuggestion) { 
        suggestionMarker = newMarker; 
    } else { 
        userMarker = newMarker; 
    } 
    const pointTime = new Date(point.time).toLocaleString('pt-BR', { timeZone: 'UTC' }); 
    syncPointInfo.textContent = `Ponto selecionado (${isSuggestion ? 'sugestão' : 'manual'}): ${pointTime} (UTC)`; 
    if (videoFile) { 
        positionSection.style.display = 'block'; 
        generateBtn.style.display = 'flex';
        updatePositionControls();
    } 
}

function handleGenerate() {
    if (!gpxFile || !videoFile || !selectedSyncPoint) {
        statusDiv.textContent = "Erro: Por favor, selecione os dois ficheiros e um ponto de sincronização.";
        return;
    }
    
    generateBtn.disabled = true;
    statusDiv.textContent = 'A enviar ficheiros...';
    progressContainer.style.display = 'block';
    progressBar.style.width = '0%';
    progressBar.textContent = '0%';
    logsContainer.style.display = 'none';
    downloadDiv.style.display = 'none';
    
    const formData = new FormData();
    formData.append('gpxFile', gpxFile);
    formData.append('videoFile', videoFile);
    formData.append('syncTimestamp', selectedSyncPoint.time.toISOString());

    formData.append('addSpeedoOverlay', speedoCheckbox.checked);
    if (speedoCheckbox.checked) {
        formData.append('speedoPosition', document.querySelector('input[name="speedoPosition"]:checked').value);
    }
    
    formData.append('addTrackOverlay', trackCheckbox.checked);
    if (trackCheckbox.checked) {
        formData.append('trackPosition', document.querySelector('input[name="trackPosition"]:checked').value);
    }

    const xhr = new XMLHttpRequest();
    xhr.open('POST', '/process', true);
    
    xhr.upload.onprogress = (event) => {
        if (event.lengthComputable) {
            const percentComplete = Math.round((event.loaded / event.total) * 100);
            progressBar.style.width = percentComplete + '%';
            progressBar.textContent = percentComplete + '%';
        }
    };
    
    xhr.onload = () => {
        progressContainer.style.display = 'none';
        const result = JSON.parse(xhr.responseText);
        logsPre.textContent = result.logs.join('\n');
        logsContainer.style.display = 'block';
        if (xhr.status >= 200 && xhr.status < 300) {
            statusDiv.textContent = 'Sucesso! O seu vídeo está pronto.';
            if (result.download_url) {
                downloadLink.href = result.download_url;
                downloadLink.textContent = '📥 Descarregar Vídeo Final';
                downloadDiv.style.display = 'block';
            }
        } else {
            statusDiv.textContent = `Erro: ${result.message || 'Ocorreu um erro no servidor.'}`;
        }
        generateBtn.disabled = false;
    };
    
    xhr.onerror = () => {
        statusDiv.textContent = 'Erro de rede ao enviar os ficheiros.';
        generateBtn.disabled = false;
        progressContainer.style.display = 'none';
    };
    
    xhr.send(formData);
}
