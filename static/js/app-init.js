// js/app-init.js - Inicialização da aplicação

// Elementos da UI
const gpxInput = document.getElementById('gpx-file');
const videoInput = document.getElementById('video-file');
const generateBtn = document.getElementById('generate-btn');
const logsContainer = document.getElementById('logs-container');
const logsPre = document.getElementById('logs');
const downloadBtn = document.getElementById('download-btn');
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

// Event Listeners principais
gpxInput.addEventListener('change', handleGpxUpload);
videoInput.addEventListener('change', handleVideoUpload);

// Event Listeners de idioma
document.getElementById('lang-pt').addEventListener('click', () => setLanguage('pt-BR'));
document.getElementById('lang-en').addEventListener('click', () => setLanguage('en'));

// Event Listeners do Modal
settingsBtn.addEventListener('click', () => settingsModal.classList.remove('hidden'));
closeModalBtn.addEventListener('click', () => settingsModal.classList.add('hidden'));
settingsModal.addEventListener('click', (e) => { if (e.target === settingsModal) { settingsModal.classList.add('hidden'); } });
interpolationSlider.addEventListener('input', () => { interpolationValue.textContent = `${interpolationSlider.value}s`; });

function initializeInlineOverlaySystem() {
    inlineOverlayManager = new InlineOverlayManager();
}

function initializeStravaIntegration() {
    stravaManager = new StravaManager();
}

// Inicialização quando DOM carregado
document.addEventListener('DOMContentLoaded', () => {
    setLanguage(currentLang);
    
    setTimeout(() => {
        initializeInlineOverlaySystem();
        initializeStravaIntegration(); // NOVO: Inicializar Strava
        
        if (generateBtn) {
            generateBtn.addEventListener('click', handleGenerateWithInlineOverlays);
        }
        
        validateGenerateButton();
    }, 500);
});