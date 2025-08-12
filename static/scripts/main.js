import { t, setLanguage, getCurrentLang } from './modules/i18n.js';
import NotificationManager from './modules/notificationManager.js';
import LoadingManager from './modules/loadingManager.js';
import * as mapManager from './modules/mapManager.js';
import InlineOverlayManager from './modules/overlayManager.js';
import { fetchSuggestion, generateVideo } from './modules/api.js';

// --- ELEMENTOS DA UI ---
const gpxInput = document.getElementById('gpx-file');
const videoInput = document.getElementById('video-file');
const generateBtn = document.getElementById('generate-btn');
const downloadBtn = document.getElementById('download-btn');
const settingsBtn = document.getElementById('settings-btn');
const settingsModal = document.getElementById('settings-modal');
const closeModalBtn = document.getElementById('close-modal-btn');
const interpolationSlider = document.getElementById('interpolation-slider');
const interpolationValue = document.getElementById('interpolation-value');
const syncPointInfo = document.getElementById('sync-point-info');
const gpxInfo = document.getElementById('gpx-info');
const videoInfo = document.getElementById('video-info');
const mapSection = document.getElementById('map-section');
const positionSection = document.getElementById('position-section');
const logsContainer = document.getElementById('logs-container');
const logsPre = document.getElementById('logs');

// --- INSTÂNCIAS DOS MÓDULOS ---
const notify = new NotificationManager();
const loader = new LoadingManager();
let inlineOverlayManager;

// --- ESTADO DA APLICAÇÃO ---
let state = {
    gpxFile: null,
    videoFile: null,
    selectedSyncPoint: null,
    gpxDataPoints: [],
};

// --- FUNÇÕES DE LÓGICA PRINCIPAL ---

function validateGenerateButton() {
    const { gpxFile, videoFile, selectedSyncPoint } = state;
    const hasFiles = gpxFile && videoFile;
    const hasSyncPoint = selectedSyncPoint !== null;
    const hasActiveOverlay = inlineOverlayManager ? inlineOverlayManager.hasActiveOverlays() : false;
    
    const canGenerate = hasFiles && hasSyncPoint && hasActiveOverlay;
    generateBtn.disabled = !canGenerate;
    generateBtn.style.display = (hasFiles && hasSyncPoint) ? 'flex' : 'none';
}

function selectSyncPoint(point, isSuggestion) {
    state.selectedSyncPoint = { ...point, time: new Date(point.time) };
    mapManager.placeSyncMarker(point, isSuggestion);

    const pointTime = point.displayTime || new Date(point.time).toLocaleString(getCurrentLang().startsWith('en') ? 'en-US' : 'pt-BR', { timeZone: 'UTC' });
    syncPointInfo.textContent = t('sync_point_selected', { 
        type: isSuggestion ? t('suggestion_type') : t('manual_type'), 
        time: pointTime 
    }); 
    
    if (!isSuggestion) notify.success(t('notification_sync_selected'), pointTime);
    
    if (state.videoFile) positionSection.style.display = 'block';
    validateGenerateButton();
}

async function handleFileChange() {
    if (!state.gpxFile || !state.videoFile) return;

    const data = await fetchSuggestion({ ...state, interpolationSlider, notify, t });
    if (data) {
        if (data.interpolated_points) {
            state.gpxDataPoints = data.interpolated_points.map(p => ({ ...p, time: new Date(p.time) }));
            mapManager.displayTrack(state.gpxDataPoints, selectSyncPoint);
        }
        if (data.timestamp) {
            const suggestedPoint = { lat: data.latitude, lon: data.longitude, time: new Date(data.timestamp), displayTime: data.display_timestamp };
            selectSyncPoint(suggestedPoint, true);
        }
    }
}

function handleGenerateClick() {
    generateBtn.disabled = true;
    loader.show(() => {
        notify.warning('Cancelamento', 'Processamento cancelado pelo usuário');
        validateGenerateButton();
    });

    const context = {
        ...state,
        interpolationSlider,
        inlineOverlayManager,
        loader,
        notify,
        t,
        lang: getCurrentLang(),
        onFinish: (successResult, errorResult) => {
            validateGenerateButton();
            logsContainer.style.display = 'block';
            if (successResult) {
                logsPre.textContent = successResult.logs.join('\n');
                if (successResult.download_url) {
                    downloadBtn.href = successResult.download_url;
                    downloadBtn.classList.add('show');
                }
            } else {
                logsPre.textContent = errorResult.logs ? errorResult.logs.join('\n') : errorResult.message;
            }
        }
    };
    generateVideo(context);
}

// --- INICIALIZAÇÃO E EVENT LISTENERS ---

document.addEventListener('DOMContentLoaded', () => {
    // Inicializar Idioma
    setLanguage(getCurrentLang(), { downloadBtn, settingsBtn });
    document.getElementById('lang-pt').addEventListener('click', () => setLanguage('pt-BR', { downloadBtn, settingsBtn }));
    document.getElementById('lang-en').addEventListener('click', () => setLanguage('en', { downloadBtn, settingsBtn }));

    // Inicializar Mapa
    mapManager.initializeMap('map');

    // Inicializar Gerenciador de Overlays
    inlineOverlayManager = new InlineOverlayManager(t, validateGenerateButton);

    // Eventos de Inputs
    gpxInput.addEventListener('change', (e) => {
        state.gpxFile = e.target.files[0];
        if (!state.gpxFile) return;
        gpxInfo.textContent = state.gpxFile.name;
        notify.success(t('notification_gpx_loaded'), t('gpx_loaded'));
        videoInput.disabled = false;
        videoInfo.textContent = t('can_select_video');
        validateGenerateButton();
        handleFileChange();
    });

    videoInput.addEventListener('change', (e) => {
        state.videoFile = e.target.files[0];
        if (!state.videoFile) return;
        videoInfo.textContent = state.videoFile.name;
        notify.success(t('notification_video_loaded'), state.videoFile.name);
        mapSection.style.display = 'block';
        mapManager.invalidateMapSize();
        validateGenerateButton();
        handleFileChange();
    });

    // Eventos de Botões e Modal
    generateBtn.addEventListener('click', handleGenerateClick);
    settingsBtn.addEventListener('click', () => settingsModal.classList.remove('hidden'));
    closeModalBtn.addEventListener('click', () => settingsModal.classList.add('hidden'));
    settingsModal.addEventListener('click', (e) => { if (e.target === settingsModal) settingsModal.classList.add('hidden'); });
    interpolationSlider.addEventListener('input', () => { interpolationValue.textContent = `${interpolationSlider.value}s`; });

    validateGenerateButton();
});