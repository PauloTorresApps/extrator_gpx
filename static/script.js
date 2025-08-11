// --- Seção de Tradução (com adição do overlay de Stats) ---
const translations = {
    'en': {
        'main_title': '🎬 Interactive GPX + Video Sync',
        'intro_text': 'Upload your files, select a sync point on the map, and configure the overlays to generate your final video with telemetry.',
        'step1_title': 'Select Files', 'gpx_file_label': 'GPX File', 'choose_gpx': 'Choose GPX', 'no_gpx_selected': 'No file selected', 'video_file_label': 'Video File', 'choose_video': 'Choose Video', 'select_gpx_first': 'Select a GPX file first',
        'step2_title': 'Select Sync Point', 'map_click_prompt': '🎯 Click a point on the map to set it as the sync start.', 'step3_title': 'Positioning', 'speedo_label': '⚙️ Speedometer', 'map_label': '🗺️ Track Map', 'stats_label': '📊 Statistics',
        'generate_button': 'Confirm and Generate Video', 'status_initial': 'Select a GPX file to begin.', 'download_link': '📥 Download Final Video', 'logs_title': '📋 Processing Logs:',
        'gpx_loaded': 'GPX file loaded. Please select the video file.', 'can_select_video': 'You can now select the video file', 'analyzing_files': 'Analyzing files to suggest sync point and track...', 'high_precision_track_loaded': 'High-precision track loaded from server.',
        'suggestion_applied': 'Automatic suggestion applied! You can adjust it on the map if needed.', 'suggestion_error': 'Could not get suggestion: {{message}}. Please select a point manually.', 'suggestion_comm_error': 'Communication error while getting suggestion. Please select a point manually.',
        'sync_point_selected': 'Point selected ({{type}}): {{time}} (UTC)', 'manual_type': 'manual', 'suggestion_type': 'suggestion', 'error_missing_files': 'Error: Please select both files and a sync point.',
        'uploading_files': 'Uploading files...', 'success_message': 'Success! Your video is ready.', 'server_error': 'Error: {{message}}', 'network_error': 'Network error while uploading files.',
        'settings_title': 'Advanced Settings', 'interpolation_label': 'Interpolation Precision Level', 'interpolation_desc': 'Lower value = more points = higher precision and slower processing.'
    },
    'pt-BR': {
        'main_title': '🎬 Sincronização Interativa GPX + Vídeo',
        'intro_text': 'Carregue os seus ficheiros, selecione um ponto de sincronização no mapa e configure os overlays para gerar o seu vídeo final com telemetria.',
        'step1_title': 'Selecionar Ficheiros', 'gpx_file_label': 'Ficheiro GPX', 'choose_gpx': 'Escolher GPX', 'no_gpx_selected': 'Nenhum ficheiro selecionado', 'video_file_label': 'Ficheiro de Vídeo', 'choose_video': 'Escolher Vídeo', 'select_gpx_first': 'Selecione um ficheiro GPX primeiro',
        'step2_title': 'Selecionar Ponto de Sincronização', 'map_click_prompt': '🎯 Clique num ponto no mapa para o definir como o início da sincronização.', 'step3_title': 'Posicionamento', 'speedo_label': '⚙️ Velocímetro', 'map_label': '🗺️ Mapa do Trajeto', 'stats_label': '📊 Estatísticas',
        'generate_button': 'Confirmar e Gerar Vídeo', 'status_initial': 'Selecione um ficheiro GPX para começar.', 'download_link': '📥 Descarregar Vídeo Final', 'logs_title': '📋 Logs do Processamento:',
        'gpx_loaded': 'Ficheiro GPX carregado. Selecione o ficheiro de vídeo.', 'can_select_video': 'Agora pode selecionar o ficheiro de vídeo', 'analyzing_files': 'Analisando ficheiros para sugerir ponto e percurso...', 'high_precision_track_loaded': 'Percurso de alta precisão carregado do servidor.',
        'suggestion_applied': 'Sugestão automática aplicada! Pode ajustar no mapa se necessário.', 'suggestion_error': 'Não foi possível obter sugestão: {{message}}. Selecione um ponto manualmente.', 'suggestion_comm_error': 'Erro de comunicação ao obter sugestão. Selecione um ponto manualmente.',
        'sync_point_selected': 'Ponto selecionado ({{type}}): {{time}} (UTC)', 'manual_type': 'manual', 'suggestion_type': 'sugestão', 'error_missing_files': 'Erro: Por favor, selecione os dois ficheiros e um ponto de sincronização.',
        'uploading_files': 'A enviar ficheiros...', 'success_message': 'Sucesso! O seu vídeo está pronto.', 'server_error': 'Erro: {{message}}', 'network_error': 'Erro de rede ao enviar os ficheiros.',
        'settings_title': 'Configurações Avançadas', 'interpolation_label': 'Nível de Precisão da Interpolação', 'interpolation_desc': 'Menor valor = mais pontos = maior precisão e processamento mais lento.'
    }
};
let currentLang = localStorage.getItem('lang') || 'pt-BR';
function t(key, replacements = {}) { let text = translations[currentLang][key] || key; for (const placeholder in replacements) { text = text.replace(`{{${placeholder}}}`, replacements[placeholder]); } return text; }
function setLanguage(lang) {
    currentLang = lang;
    localStorage.setItem('lang', lang);
    document.documentElement.lang = lang.startsWith('en') ? 'en' : 'pt-BR';
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        const hasChildElements = el.children.length > 0 && Array.from(el.children).some(child => child.nodeType === 1);
        if (hasChildElements) {
            const textNode = Array.from(el.childNodes).find(node => node.nodeType === Node.TEXT_NODE && node.textContent.trim().length > 0);
            if(textNode) textNode.textContent = t(key);
        } else {
            el.textContent = t(key);
        }
    });
    document.getElementById('lang-pt').classList.toggle('active', lang === 'pt-BR');
    document.getElementById('lang-en').classList.toggle('active', lang === 'en');
}

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
const speedoCheckbox = document.getElementById('add-speedo-overlay');
const trackCheckbox = document.getElementById('add-track-overlay');
const statsCheckbox = document.getElementById('add-stats-overlay');
const speedoPositionGrid = document.getElementById('speedo-position-grid');
const trackPositionGrid = document.getElementById('track-position-grid');
const statsPositionGrid = document.getElementById('stats-position-grid');
const speedoPositionRadios = document.querySelectorAll('input[name="speedoPosition"]');
const trackPositionRadios = document.querySelectorAll('input[name="trackPosition"]');
const statsPositionRadios = document.querySelectorAll('input[name="statsPosition"]');

// Seletores para o Modal
const settingsBtn = document.getElementById('settings-btn');
const settingsModal = document.getElementById('settings-modal');
const closeModalBtn = document.getElementById('close-modal-btn');
const interpolationSlider = document.getElementById('interpolation-slider');
const interpolationValue = document.getElementById('interpolation-value');

// Variáveis de estado
let gpxFile = null, videoFile = null, selectedSyncPoint = null, map = null, trackLayer = null, userMarker = null, suggestionMarker = null, gpxDataPoints = [];

// Inicialização do mapa
map = L.map('map').setView([0, 0], 2);
L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', { attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors' }).addTo(map);

// Event Listeners
gpxInput.addEventListener('change', handleGpxUpload);
videoInput.addEventListener('change', handleVideoUpload);
generateBtn.addEventListener('click', handleGenerate);
speedoCheckbox.addEventListener('change', () => { speedoPositionGrid.classList.toggle('hidden', !speedoCheckbox.checked); updatePositionControls(); });
trackCheckbox.addEventListener('change', () => { trackPositionGrid.classList.toggle('hidden', !trackCheckbox.checked); updatePositionControls(); });
statsCheckbox.addEventListener('change', () => { statsPositionGrid.classList.toggle('hidden', !statsCheckbox.checked); updatePositionControls(); });
speedoPositionRadios.forEach(radio => radio.addEventListener('change', updatePositionControls));
trackPositionRadios.forEach(radio => radio.addEventListener('change', updatePositionControls));
statsPositionRadios.forEach(radio => radio.addEventListener('change', updatePositionControls));
document.getElementById('lang-pt').addEventListener('click', () => setLanguage('pt-BR'));
document.getElementById('lang-en').addEventListener('click', () => setLanguage('en'));

// Event Listeners do Modal
settingsBtn.addEventListener('click', () => settingsModal.classList.remove('hidden'));
closeModalBtn.addEventListener('click', () => settingsModal.classList.add('hidden'));
settingsModal.addEventListener('click', (e) => { if (e.target === settingsModal) { settingsModal.classList.add('hidden'); } });
interpolationSlider.addEventListener('input', () => { interpolationValue.textContent = `${interpolationSlider.value}s`; });

function updatePositionControls() {
    const speedoPos = speedoCheckbox.checked ? document.querySelector('input[name="speedoPosition"]:checked').value : null;
    const trackPos = trackCheckbox.checked ? document.querySelector('input[name="trackPosition"]:checked').value : null;
    const statsPos = statsCheckbox.checked ? document.querySelector('input[name="statsPosition"]:checked').value : null;
    
    const usedPositions = [speedoPos, trackPos, statsPos].filter(pos => pos !== null);
    
    trackPositionRadios.forEach(radio => { 
        radio.disabled = usedPositions.includes(radio.value) && radio.value !== trackPos; 
    });
    speedoPositionRadios.forEach(radio => { 
        radio.disabled = usedPositions.includes(radio.value) && radio.value !== speedoPos; 
    });
    statsPositionRadios.forEach(radio => { 
        radio.disabled = usedPositions.includes(radio.value) && radio.value !== statsPos; 
    });
}

async function fetchAndApplySuggestion() {
    if (!gpxFile || !videoFile) return;
    statusDiv.textContent = t('analyzing_files');
    const formData = new FormData();
    formData.append('gpxFile', gpxFile);
    formData.append('videoFile', videoFile);
    formData.append('interpolationLevel', interpolationSlider.value);
    try {
        const response = await fetch('/suggest', { method: 'POST', body: formData });
        const data = await response.json();
        if (data.interpolated_points && data.interpolated_points.length > 0) {
            statusDiv.textContent = t('high_precision_track_loaded');
            gpxDataPoints = data.interpolated_points.map(p => ({ ...p, time: new Date(p.time) }));
            displayTrack(gpxDataPoints);
        }
        if (response.ok && data.timestamp) {
            const suggestedPoint = { lat: data.latitude, lon: data.longitude, time: new Date(data.timestamp) };
            selectSyncPoint(suggestedPoint, true);
            statusDiv.textContent = t('suggestion_applied');
        } else {
            if (!data.interpolated_points) { statusDiv.textContent = t('suggestion_error', { message: data.message || 'Unknown error' }); }
        }
    } catch (error) {
        console.error("Error calling /suggest endpoint:", error);
        statusDiv.textContent = t('suggestion_comm_error');
    }
}

const suggestionIcon = L.icon({ iconUrl: 'data:image/svg+xml;base64,' + btoa(`<svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41"><path fill="#2E8B57" stroke="#FFFFFF" stroke-width="1.5" d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/><circle fill="#FFFFFF" cx="12.5" cy="12.5" r="4"/></svg>`), iconSize: [25, 41], iconAnchor: [12, 41], popupAnchor: [1, -34] });
const userIcon = L.icon({ iconUrl: 'data:image/svg+xml;base64,' + btoa(`<svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41"><path fill="#03dac6" stroke="#121212" stroke-width="1" d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/><circle fill="#121212" cx="12.5" cy="12.5" r="4"/></svg>`), iconSize: [25, 41], iconAnchor: [12, 41], popupAnchor: [1, -34] });

function handleGpxUpload(event) { gpxFile = event.target.files[0]; if (!gpxFile) return; gpxInfo.textContent = gpxFile.name; statusDiv.textContent = t('gpx_loaded'); videoInput.disabled = false; videoInfo.textContent = t('can_select_video'); fetchAndApplySuggestion(); }
function handleVideoUpload(event) { videoFile = event.target.files[0]; if (!videoFile) return; videoInfo.textContent = videoFile.name; checkAndShowMapSection(); fetchAndApplySuggestion(); }
function checkAndShowMapSection() { if (gpxFile && videoFile) { mapSection.style.display = 'block'; syncPointInfo.style.display = 'block'; setTimeout(() => map.invalidateSize(), 100); } }
function displayTrack(points) { if (trackLayer) map.removeLayer(trackLayer); const latLngs = points.map(p => [p.lat, p.lon]); trackLayer = L.polyline(latLngs, { color: '#bb86fc', weight: 3, opacity: 0.8 }).addTo(map); const bounds = trackLayer.getBounds(); if (bounds.isValid()) { map.fitBounds(bounds.pad(0.1)); } trackLayer.on('click', (e) => { let closestPoint = null, minDistance = Infinity; gpxDataPoints.forEach(p => { const distance = map.distance([p.lat, p.lon], e.latlng); if (distance < minDistance) { minDistance = distance; closestPoint = p; } }); if (closestPoint) { selectSyncPoint(closestPoint, false); } }); }
function selectSyncPoint(point, isSuggestion) { selectedSyncPoint = point; if (userMarker) map.removeLayer(userMarker); if (suggestionMarker) map.removeLayer(suggestionMarker); const iconToUse = isSuggestion ? suggestionIcon : userIcon; const newMarker = L.marker([point.lat, point.lon], { icon: iconToUse }).addTo(map); if(isSuggestion) { suggestionMarker = newMarker; } else { userMarker = newMarker; } const pointTime = new Date(point.time).toLocaleString(currentLang.startsWith('en') ? 'en-US' : 'pt-BR', { timeZone: 'UTC' }); syncPointInfo.textContent = t('sync_point_selected', { type: isSuggestion ? t('suggestion_type') : t('manual_type'), time: pointTime }); if (videoFile) { positionSection.style.display = 'block'; generateBtn.style.display = 'flex'; updatePositionControls(); } }

function handleGenerate() {
    if (!gpxFile || !videoFile || !selectedSyncPoint) { statusDiv.textContent = t('error_missing_files'); return; }
    generateBtn.disabled = true;
    statusDiv.textContent = t('uploading_files');
    progressContainer.style.display = 'block';
    progressBar.style.width = '0%';
    progressBar.textContent = '0%';
    logsContainer.style.display = 'none';
    downloadDiv.style.display = 'none';
    
    const formData = new FormData();
    formData.append('gpxFile', gpxFile);
    formData.append('videoFile', videoFile);
    formData.append('syncTimestamp', selectedSyncPoint.time.toISOString());
    formData.append('lang', currentLang);
    formData.append('interpolationLevel', interpolationSlider.value);

    formData.append('addSpeedoOverlay', speedoCheckbox.checked);
    if (speedoCheckbox.checked) { formData.append('speedoPosition', document.querySelector('input[name="speedoPosition"]:checked').value); }
    formData.append('addTrackOverlay', trackCheckbox.checked);
    if (trackCheckbox.checked) { formData.append('trackPosition', document.querySelector('input[name="trackPosition"]:checked').value); }
    formData.append('addStatsOverlay', statsCheckbox.checked);
    if (statsCheckbox.checked) { formData.append('statsPosition', document.querySelector('input[name="statsPosition"]:checked').value); }

    const xhr = new XMLHttpRequest();
    xhr.open('POST', '/process', true);
    xhr.upload.onprogress = (event) => { if (event.lengthComputable) { const percentComplete = Math.round((event.loaded / event.total) * 100); progressBar.style.width = percentComplete + '%'; progressBar.textContent = percentComplete + '%'; } };
    xhr.onload = () => {
        progressContainer.style.display = 'none';
        const result = JSON.parse(xhr.responseText);
        logsPre.textContent = result.logs.join('\n');
        logsContainer.style.display = 'block';
        if (xhr.status >= 200 && xhr.status < 300) {
            statusDiv.textContent = t('success_message');
            if (result.download_url) { downloadLink.href = result.download_url; downloadDiv.style.display = 'block'; }
        } else {
            statusDiv.textContent = t('server_error', { message: result.message || 'Unknown error' });
        }
        generateBtn.disabled = false;
    };
    xhr.onerror = () => { statusDiv.textContent = t('network_error'); generateBtn.disabled = false; progressContainer.style.display = 'none'; };
    xhr.send(formData);
}

document.addEventListener('DOMContentLoaded', () => { setLanguage(currentLang); });