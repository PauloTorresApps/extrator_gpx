// --- Seção de Tradução (com adição do overlay de Stats) ---
const translations = {
    'en': {
        'main_title': '🎬 Interactive GPX + Video Sync',
        'intro_text': 'Upload your files, select a sync point on the map, and configure the overlays to generate your final video with telemetry.',
        'step1_title': 'Select Files', 'gpx_file_label': 'GPX File', 'choose_gpx': 'Choose GPX', 'no_gpx_selected': 'No file selected', 'video_file_label': 'Video File', 'choose_video': 'Choose Video', 'select_gpx_first': 'Select a GPX file first',
        'step2_title': 'Select Sync Point', 'map_click_prompt': '🎯 Click a point on the map to set it as the sync start.', 'step3_title': 'Positioning', 'speedo_label': '⏱️ Speedometer', 'map_label': '🗺️ Track Map', 'stats_label': '📊 Statistics',
        'generate_button': 'Confirm and Generate Video', 'download_link': '📥 Download Final Video', 'logs_title': '📋 Processing Logs:',
        'gpx_loaded': 'GPX file loaded successfully', 'can_select_video': 'You can now select the video file', 'analyzing_files': 'Analyzing files to suggest sync point and track...', 'high_precision_track_loaded': 'High-precision track loaded from server.',
        'suggestion_applied': 'Automatic suggestion applied! You can adjust it on the map if needed.', 'suggestion_error': 'Could not get suggestion: {{message}}. Please select a point manually.', 'suggestion_comm_error': 'Communication error while getting suggestion. Please select a point manually.',
        'sync_point_selected': 'Point selected ({{type}}): {{time}} (UTC)', 'manual_type': 'manual', 'suggestion_type': 'suggestion', 'error_missing_files': 'Error: Please select both files and a sync point.',
        'uploading_files': 'Uploading files...', 'success_message': 'Success! Your video is ready.', 'server_error': 'Error: {{message}}', 'network_error': 'Network error while uploading files.',
        'settings_title': 'Advanced Settings', 'interpolation_label': 'Interpolation Precision Level', 'interpolation_desc': 'Lower value = more points = higher precision and slower processing.',
        'speedo_hint': 'Displays a speedometer with the current speed on the video.',
        'map_hint': 'Shows a mini-map with the traveled path and current position.',
        'stats_hint': 'Adds a panel with statistics like distance, time, and elevation.',
        'error_missing_overlay': 'Error: Please select at least one overlay (Speedometer, Map, or Statistics).',
        // Notificações
        'notification_gpx_loaded': 'GPX Loaded',
        'notification_video_loaded': 'Video Loaded', 
        'notification_sync_selected': 'Sync Point Selected',
        'notification_processing': 'Processing Video',
        'notification_success': 'Video Ready',
        'notification_error': 'Error',
        'notification_suggestion': 'Auto Suggestion'
    },
    'pt-BR': {
        'main_title': '🎬 Sincronização Interativa GPX + Vídeo',
        'intro_text': 'Carregue os seus ficheiros, selecione um ponto de sincronização no mapa e configure os overlays para gerar o seu vídeo final com telemetria.',
        'step1_title': 'Selecionar Ficheiros', 'gpx_file_label': 'Ficheiro GPX', 'choose_gpx': 'Escolher GPX', 'no_gpx_selected': 'Nenhum ficheiro selecionado', 'video_file_label': 'Ficheiro de Vídeo', 'choose_video': 'Escolher Vídeo', 'select_gpx_first': 'Selecione um ficheiro GPX primeiro',
        'step2_title': 'Selecionar Ponto de Sincronização', 'map_click_prompt': '🎯 Clique num ponto no mapa para o definir como o início da sincronização.', 'step3_title': 'Posicionamento', 'speedo_label': '⚙️ Velocímetro', 'map_label': '🗺️ Mapa do Trajeto', 'stats_label': '📊 Estatísticas',
        'generate_button': 'Confirmar e Gerar Vídeo', 'download_link': '📥 Descarregar Vídeo Final', 'logs_title': '📋 Logs do Processamento:',
        'gpx_loaded': 'Ficheiro GPX carregado com sucesso', 'can_select_video': 'Agora pode selecionar o ficheiro de vídeo', 'analyzing_files': 'Analisando ficheiros para sugerir ponto e percurso...', 'high_precision_track_loaded': 'Percurso de alta precisão carregado do servidor.',
        'suggestion_applied': 'Sugestão automática aplicada! Pode ajustar no mapa se necessário.', 'suggestion_error': 'Não foi possível obter sugestão: {{message}}. Selecione um ponto manualmente.', 'suggestion_comm_error': 'Erro de comunicação ao obter sugestão. Selecione um ponto manualmente.',
        'sync_point_selected': 'Ponto selecionado ({{type}}): {{time}} (UTC)', 'manual_type': 'manual', 'suggestion_type': 'sugestão', 'error_missing_files': 'Erro: Por favor, selecione os dois ficheiros e um ponto de sincronização.',
        'uploading_files': 'A enviar ficheiros...', 'success_message': 'Sucesso! O seu vídeo está pronto.', 'server_error': 'Erro: {{message}}', 'network_error': 'Erro de rede ao enviar os ficheiros.',
        'settings_title': 'Configurações Avançadas', 'interpolation_label': 'Nível de Precisão da Interpolação', 'interpolation_desc': 'Menor valor = mais pontos = maior precisão e processamento mais lento.',
        'speedo_hint': 'Exibe um velocímetro com a velocidade atual no vídeo.',
        'map_hint': 'Mostra um mini-mapa com o trajeto percorrido e a posição atual.',
        'stats_hint': 'Adiciona um painel com estatísticas como distância, tempo e elevação.',
        'error_missing_overlay': 'Erro: Por favor, selecione pelo menos um overlay (Velocímetro, Mapa ou Estatísticas).',
        // Notificações
        'notification_gpx_loaded': 'GPX Carregado',
        'notification_video_loaded': 'Vídeo Carregado',
        'notification_sync_selected': 'Ponto Selecionado',
        'notification_processing': 'Processando Vídeo', 
        'notification_success': 'Vídeo Pronto',
        'notification_error': 'Erro',
        'notification_suggestion': 'Sugestão Automática'
    }
};

let currentLang = localStorage.getItem('lang') || 'pt-BR';

function t(key, replacements = {}) { 
    let text = translations[currentLang][key] || key; 
    for (const placeholder in replacements) { 
        text = text.replace(`{{${placeholder}}}`, replacements[placeholder]); 
    } 
    return text; 
}

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

// === SISTEMA DE NOTIFICAÇÕES ===
class NotificationManager {
    constructor() {
        this.container = document.getElementById('notifications-container');
        this.notifications = new Map();
        this.notificationCounter = 0;
    }

    show(type, title, message, options = {}) {
        const id = `notification-${++this.notificationCounter}`;
        const duration = options.duration || 5000;
        const persistent = options.persistent || false;

        // Ícones por tipo
        const icons = {
            success: '✅',
            error: '❌', 
            warning: '⚠️',
            info: 'ℹ️'
        };

        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.id = id;
        notification.innerHTML = `
            <div class="notification-header">
                <div class="notification-icon">${icons[type] || '📋'}</div>
                <div class="notification-title">${title}</div>
                <button class="notification-close" aria-label="Fechar">&times;</button>
            </div>
            ${message ? `<div class="notification-message">${message}</div>` : ''}
            ${!persistent ? '<div class="notification-progress"></div>' : ''}
        `;

        // Event listener para fechar
        const closeBtn = notification.querySelector('.notification-close');
        closeBtn.addEventListener('click', () => this.hide(id));

        // Adicionar ao container
        this.container.appendChild(notification);
        this.notifications.set(id, notification);

        // Mostrar com animação
        setTimeout(() => notification.classList.add('show'), 50);

        // Auto-hide se não for persistente
        if (!persistent && duration > 0) {
            setTimeout(() => this.hide(id), duration);
        }

        return id;
    }

    hide(id) {
        const notification = this.notifications.get(id);
        if (notification) {
            notification.classList.add('hide');
            setTimeout(() => {
                if (notification.parentNode) {
                    notification.parentNode.removeChild(notification);
                }
                this.notifications.delete(id);
            }, 400);
        }
    }

    success(title, message, options = {}) {
        return this.show('success', title, message, options);
    }

    error(title, message, options = {}) {
        return this.show('error', title, message, { ...options, persistent: true });
    }

    warning(title, message, options = {}) {
        return this.show('warning', title, message, options);
    }

    info(title, message, options = {}) {
        return this.show('info', title, message, options);
    }

    clearAll() {
        Array.from(this.notifications.keys()).forEach(id => this.hide(id));
    }
}

// Instância global do gerenciador de notificações
const notify = new NotificationManager();

// Elementos da UI
const gpxInput = document.getElementById('gpx-file');
const videoInput = document.getElementById('video-file');
const generateBtn = document.getElementById('generate-btn');
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
L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', { 
    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors' 
}).addTo(map);

// Event Listeners
gpxInput.addEventListener('change', handleGpxUpload);
videoInput.addEventListener('change', handleVideoUpload);

document.getElementById('lang-pt').addEventListener('click', () => setLanguage('pt-BR'));
document.getElementById('lang-en').addEventListener('click', () => setLanguage('en'));

// Event Listeners do Modal
settingsBtn.addEventListener('click', () => settingsModal.classList.remove('hidden'));
closeModalBtn.addEventListener('click', () => settingsModal.classList.add('hidden'));
settingsModal.addEventListener('click', (e) => { if (e.target === settingsModal) { settingsModal.classList.add('hidden'); } });
interpolationSlider.addEventListener('input', () => { interpolationValue.textContent = `${interpolationSlider.value}s`; });

// === NOVA FUNÇÃO: Validação de Estado do Botão Generate ===
function validateGenerateButton() {
    const hasFiles = gpxFile && videoFile;
    const hasSyncPoint = selectedSyncPoint !== null;
    const hasActiveOverlay = inlineOverlayManager ? inlineOverlayManager.hasActiveOverlays() : false;
    
    const canGenerate = hasFiles && hasSyncPoint && hasActiveOverlay;
    
    generateBtn.disabled = !canGenerate;
    
    // Atualizar visibilidade do botão baseado nos arquivos e sync point
    if (hasFiles && hasSyncPoint) {
        generateBtn.style.display = 'flex';
    } else {
        generateBtn.style.display = 'none';
    }
}

async function fetchAndApplySuggestion() {
    if (!gpxFile || !videoFile) return;
    
    notify.info(t('notification_suggestion'), t('analyzing_files'));
    
    const formData = new FormData();
    formData.append('gpxFile', gpxFile);
    formData.append('videoFile', videoFile);
    formData.append('interpolationLevel', interpolationSlider.value);
    
    try {
        const response = await fetch('/suggest', { method: 'POST', body: formData });
        const data = await response.json();
        
        if (data.interpolated_points && data.interpolated_points.length > 0) {
            notify.success(t('notification_suggestion'), t('high_precision_track_loaded'));
            gpxDataPoints = data.interpolated_points.map(p => ({ ...p, time: new Date(p.time) }));
            displayTrack(gpxDataPoints);
        }
        
        if (response.ok && data.timestamp) {
            const suggestedPoint = { 
                lat: data.latitude, 
                lon: data.longitude, 
                time: new Date(data.timestamp), 
                displayTime: data.display_timestamp 
            };
            selectSyncPoint(suggestedPoint, true);
            notify.success(t('notification_suggestion'), t('suggestion_applied'));
        } else {
            if (!data.interpolated_points) { 
                notify.error(t('notification_error'), t('suggestion_error', { message: data.message || 'Unknown error' }));
            }
        }
    } catch (error) {
        console.error("Error calling /suggest endpoint:", error);
        notify.error(t('notification_error'), t('suggestion_comm_error'));
    }
}

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

function handleGpxUpload(event) { 
    gpxFile = event.target.files[0]; 
    if (!gpxFile) return; 
    
    gpxInfo.textContent = gpxFile.name; 
    notify.success(t('notification_gpx_loaded'), t('gpx_loaded'));
    
    videoInput.disabled = false; 
    videoInfo.textContent = t('can_select_video'); 
    validateGenerateButton(); 
    fetchAndApplySuggestion(); 
}

function handleVideoUpload(event) { 
    videoFile = event.target.files[0]; 
    if (!videoFile) return; 
    
    videoInfo.textContent = videoFile.name; 
    notify.success(t('notification_video_loaded'), videoFile.name);
    
    checkAndShowMapSection(); 
    validateGenerateButton(); 
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

// ==============================================
// SISTEMA INLINE DE OVERLAYS - VERSÃO FINAL CORRIGIDA
// ==============================================
class InlineOverlayManager {
    constructor() {
        this.overlayPositions = new Map();
        this.overlayStates = new Map();
        
        this.cornerSequence = ['bottom-left', 'top-left', 'top-right', 'bottom-right'];
        
        this.cornerLabels = {
            'bottom-left': 'IE', 'top-left': 'SE', 
            'top-right': 'SD', 'bottom-right': 'ID'
        };
        
        this.overlayConfig = {
            speedometer: { icon: '⏱️', name: 'Velocímetro' },
            map: { icon: '🗺️', name: 'Mapa' },
            stats: { icon: '📊', name: 'Estatísticas' }
        };
        
        this.init();
    }
    
    init() {
        this.replaceOldSystem();
        this.addEventListeners();
        
        this.overlayStates.set('speedometer', false);
        this.overlayStates.set('map', false);
        this.overlayStates.set('stats', false);
        
        this.updateInterface();
        this.updateLegacyControls();
    }
    
    replaceOldSystem() {
        const container = document.querySelector('.overlays-config-container');
        if (!container) return;
        
        container.innerHTML = `
            <div class="overlay-images-group">
                <div class="overlay-image speedometer-img" data-overlay="speedometer" title="${t('speedo_hint')}">
                    <div class="active-indicator"></div><div class="position-indicator"></div>
                    <img width="48" height="48" src="./images/velocimetro.png">
                </div>
                <div class="overlay-image map-img" data-overlay="map" title="${t('map_hint')}">
                    <div class="active-indicator"></div><div class="position-indicator"></div>
                    <img width="48" height="48" src="./images/mapa.png">
                </div>
                <div class="overlay-image stats-img" data-overlay="stats" title="${t('stats_hint')}">
                    <div class="active-indicator"></div><div class="position-indicator"></div>
                    <img width="48" height="48" src="./images/grafico.png">
                </div>
            </div>
            <div class="overlay-separator"></div>
            <div class="corner-controls-container">
                <div class="corner-controls">
                    <div class="corner-btn corner-top-left" data-corner="top-left">↖</div>
                    <div class="corner-btn corner-top-right" data-corner="top-right">↗</div>
                    <div class="corner-btn corner-bottom-left" data-corner="bottom-left">↙</div>
                    <div class="corner-btn corner-bottom-right" data-corner="bottom-right">↘</div>
                </div>
                <div class="control-label">Clique nas imagens para ativar</div>
            </div>
        `;
    }
    
    addEventListeners() {
        document.querySelectorAll('.overlay-image').forEach(image => {
            image.addEventListener('click', (e) => {
                const overlay = e.currentTarget.dataset.overlay;
                this.handleOverlayClick(overlay);
            });
        });
    }

    handleOverlayClick(overlayType) {
        const isActive = this.overlayStates.get(overlayType);

        if (!isActive) {
            const firstAvailableCorner = this.findFirstAvailableCorner();
            if (firstAvailableCorner) {
                this.overlayStates.set(overlayType, true);
                this.overlayPositions.set(overlayType, firstAvailableCorner);
            }
        } else {
            const availableCornersForThisOverlay = this.cornerSequence.filter(corner => {
                for (const [otherOverlay, otherPosition] of this.overlayPositions.entries()) {
                    if (otherOverlay !== overlayType && otherPosition === corner) {
                        return false;
                    }
                }
                return true;
            });

            const currentPosition = this.overlayPositions.get(overlayType);
            const currentIndex = availableCornersForThisOverlay.indexOf(currentPosition);

            if (currentIndex === availableCornersForThisOverlay.length - 1) {
                this.overlayStates.set(overlayType, false);
                this.overlayPositions.delete(overlayType);
            } else {
                const nextPosition = availableCornersForThisOverlay[currentIndex + 1];
                this.overlayPositions.set(overlayType, nextPosition);
            }
        }

        this.updateInterface();
        this.updateLegacyControls();
        validateGenerateButton();
    }

    findFirstAvailableCorner() {
        const occupiedCorners = Array.from(this.overlayPositions.values());
        for (const corner of this.cornerSequence) {
            if (!occupiedCorners.includes(corner)) {
                return corner;
            }
        }
        return null;
    }
    
    updateInterface() {
        const allOverlays = Object.keys(this.overlayConfig);
        
        allOverlays.forEach(overlay => {
            const imageElement = document.querySelector(`[data-overlay="${overlay}"]`);
            const positionIndicator = imageElement.querySelector('.position-indicator');
            const isActive = this.overlayStates.get(overlay);
            
            imageElement.classList.toggle('active', isActive);
            
            if (isActive) {
                const corner = this.overlayPositions.get(overlay);
                positionIndicator.textContent = this.cornerLabels[corner] || '';
            } else {
                positionIndicator.textContent = '';
            }
        });
        
        const occupiedCorners = Array.from(this.overlayPositions.values());
        document.querySelectorAll('.corner-btn').forEach(btn => {
            const corner = btn.dataset.corner;
            btn.classList.toggle('occupied', occupiedCorners.includes(corner));
        });

        const labelElement = document.querySelector('.control-label');
        const activeOverlays = [];
        
        this.cornerSequence.forEach(corner => {
            for (const [overlay, position] of this.overlayPositions.entries()) {
                if (position === corner) {
                    activeOverlays.push(`${this.overlayConfig[overlay].icon}:${this.cornerLabels[corner]}`);
                }
            }
        });
        
        if (activeOverlays.length > 0) {
            labelElement.textContent = `Ativos: ${activeOverlays.join(' | ')}`;
            labelElement.classList.add('active');
        } else {
            labelElement.textContent = 'Clique nas imagens para ativar';
            labelElement.classList.remove('active');
        }
    }
    
    updateLegacyControls() {
        if (speedoCheckbox) speedoCheckbox.checked = this.overlayStates.get('speedometer');
        if (trackCheckbox) trackCheckbox.checked = this.overlayStates.get('map');
        if (statsCheckbox) statsCheckbox.checked = this.overlayStates.get('stats');
        
        this.updateRadioButtons('speedometer', 'speedoPosition');
        this.updateRadioButtons('map', 'trackPosition');
        this.updateRadioButtons('stats', 'statsPosition');
    }
    
    updateRadioButtons(overlayType, radioName) {
        const radios = document.querySelectorAll(`input[name="${radioName}"]`);
        if (!radios.length) return;
        
        radios.forEach(radio => radio.checked = false);
        
        if (this.overlayStates.get(overlayType)) {
            const position = this.overlayPositions.get(overlayType);
            if (position) {
                const targetRadio = document.querySelector(`input[name="${radioName}"][value="${position}"]`);
                if (targetRadio) targetRadio.checked = true;
            }
        }
    }
    
    hasActiveOverlays() {
        return Array.from(this.overlayStates.values()).some(isActive => isActive === true);
    }
    
    getConfiguration() {
        return {
            addSpeedoOverlay: this.overlayStates.get('speedometer'),
            addTrackOverlay: this.overlayStates.get('map'),
            addStatsOverlay: this.overlayStates.get('stats'),
            speedoPosition: this.overlayPositions.get('speedometer') || null,
            trackPosition: this.overlayPositions.get('map') || null,
            statsPosition: this.overlayPositions.get('stats') || null
        };
    }

    applyConfiguration(config) {
        this.overlayStates.clear();
        this.overlayPositions.clear();
        
        document.querySelectorAll('.overlay-image').forEach(img => {
            img.classList.remove('active', 'selected');
        });
        
        if (config.addSpeedoOverlay === true && config.speedoPosition) {
            this.overlayStates.set('speedometer', true);
            this.overlayPositions.set('speedometer', config.speedoPosition);
        } else {
            this.overlayStates.set('speedometer', false);
        }
        
        if (config.addTrackOverlay === true && config.trackPosition) {
            this.overlayStates.set('map', true);
            this.overlayPositions.set('map', config.trackPosition);
        } else {
            this.overlayStates.set('map', false);
        }
        
        if (config.addStatsOverlay === true && config.statsPosition) {
            this.overlayStates.set('stats', true);
            this.overlayPositions.set('stats', config.statsPosition);
        } else {
            this.overlayStates.set('stats', false);
        }
        
        this.updateInterface();
        this.updateLegacyControls();
        validateGenerateButton();
    }
}

// Instanciar o gerenciador inline
let inlineOverlayManager;

// Nova função handleGenerate para usar sistema inline
function handleGenerateWithInlineOverlays() {
    if (!gpxFile || !videoFile || !selectedSyncPoint) { 
        notify.error(t('notification_error'), t('error_missing_files'));
        return; 
    }
    
    if (!inlineOverlayManager.hasActiveOverlays()) {
        notify.error(t('notification_error'), t('error_missing_overlay'));
        return;
    }
    
    generateBtn.disabled = true;
    notify.info(t('notification_processing'), t('uploading_files'));
    
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

    const overlayConfig = inlineOverlayManager.getConfiguration();
    
    formData.append('addSpeedoOverlay', overlayConfig.addSpeedoOverlay);
    if (overlayConfig.addSpeedoOverlay) { 
        formData.append('speedoPosition', overlayConfig.speedoPosition); 
    }
    
    formData.append('addTrackOverlay', overlayConfig.addTrackOverlay);
    if (overlayConfig.addTrackOverlay) { 
        formData.append('trackPosition', overlayConfig.trackPosition); 
    }
    
    formData.append('addStatsOverlay', overlayConfig.addStatsOverlay);
    if (overlayConfig.addStatsOverlay) { 
        formData.append('statsPosition', overlayConfig.statsPosition); 
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
            notify.success(t('notification_success'), t('success_message'));
            if (result.download_url) { 
                downloadLink.href = result.download_url; 
                downloadDiv.style.display = 'block'; 
            }
        } else {
            notify.error(t('notification_error'), t('server_error', { message: result.message || 'Unknown error' }));
        }
        
        generateBtn.disabled = false;
        validateGenerateButton();
    };
    
    xhr.onerror = () => { 
        notify.error(t('notification_error'), t('network_error'));
        generateBtn.disabled = false; 
        progressContainer.style.display = 'none';
        validateGenerateButton();
    };
    
    xhr.send(formData);
}

function initializeInlineOverlaySystem() {
    inlineOverlayManager = new InlineOverlayManager();
}

// Inicialização quando DOM carregado
document.addEventListener('DOMContentLoaded', () => {
    setLanguage(currentLang);
    
    setTimeout(() => {
        initializeInlineOverlaySystem();
        
        if (generateBtn) {
            generateBtn.addEventListener('click', handleGenerateWithInlineOverlays);
        }
        
        validateGenerateButton();
    }, 500);
});