export default class InlineOverlayManager {
    constructor(t, onUpdateCallback) {
        this.t = t;
        this.onUpdate = onUpdateCallback;
        this.overlayPositions = new Map();
        this.overlayStates = new Map();
        this.cornerSequence = ['bottom-left', 'top-left', 'top-right', 'bottom-right'];
        this.cornerLabels = { 'bottom-left': 'IE', 'top-left': 'SE', 'top-right': 'SD', 'bottom-right': 'ID' };
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
    }

    replaceOldSystem() {
        const container = document.querySelector('.overlays-config-container');
        if (!container) return;
        container.innerHTML = `
            <div class="overlay-images-group">
                <div class="overlay-image speedometer-img" data-overlay="speedometer" title="${this.t('speedo_hint')}"><div class="active-indicator"></div><div class="position-indicator"></div><img width="48" height="48" src="./images/velocimetro.png"></div>
                <div class="overlay-image map-img" data-overlay="map" title="${this.t('map_hint')}"><div class="active-indicator"></div><div class="position-indicator"></div><img width="48" height="48" src="./images/mapa.png"></div>
                <div class="overlay-image stats-img" data-overlay="stats" title="${this.t('stats_hint')}"><div class="active-indicator"></div><div class="position-indicator"></div><img width="48" height="48" src="./images/grafico.png"></div>
            </div>
            <div class="overlay-separator"></div>
            <div class="corner-controls-container">
                <div class="corner-controls">
                    <div class="corner-btn corner-top-left" data-corner="top-left">↖</div><div class="corner-btn corner-top-right" data-corner="top-right">↗</div><div class="corner-btn corner-bottom-left" data-corner="bottom-left">↙</div><div class="corner-btn corner-bottom-right" data-corner="bottom-right">↘</div>
                </div>
                <div class="control-label">Clique nas imagens para ativar</div>
            </div>`;
    }

    addEventListeners() {
        document.querySelectorAll('.overlay-image').forEach(image => {
            image.addEventListener('click', (e) => this.handleOverlayClick(e.currentTarget.dataset.overlay));
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
            const availableCorners = this.cornerSequence.filter(c => !Array.from(this.overlayPositions.values()).includes(c) || this.overlayPositions.get(overlayType) === c);
            const currentPosition = this.overlayPositions.get(overlayType);
            const currentIndex = availableCorners.indexOf(currentPosition);
            if (currentIndex === availableCorners.length - 1) {
                this.overlayStates.set(overlayType, false);
                this.overlayPositions.delete(overlayType);
            } else {
                this.overlayPositions.set(overlayType, availableCorners[currentIndex + 1]);
            }
        }
        this.updateInterface();
        this.onUpdate();
    }

    findFirstAvailableCorner() {
        const occupiedCorners = Array.from(this.overlayPositions.values());
        return this.cornerSequence.find(corner => !occupiedCorners.includes(corner)) || null;
    }

    updateInterface() {
        Object.keys(this.overlayConfig).forEach(overlay => {
            const imageElement = document.querySelector(`[data-overlay="${overlay}"]`);
            const isActive = this.overlayStates.get(overlay);
            imageElement.classList.toggle('active', isActive);
            if (isActive) {
                imageElement.querySelector('.position-indicator').textContent = this.cornerLabels[this.overlayPositions.get(overlay)] || '';
            }
        });
        const occupiedCorners = Array.from(this.overlayPositions.values());
        document.querySelectorAll('.corner-btn').forEach(btn => btn.classList.toggle('occupied', occupiedCorners.includes(btn.dataset.corner)));
        const activeOverlays = this.cornerSequence.map(corner => {
            const overlay = Array.from(this.overlayPositions.keys()).find(key => this.overlayPositions.get(key) === corner);
            return overlay ? `${this.overlayConfig[overlay].icon}:${this.cornerLabels[corner]}` : null;
        }).filter(Boolean);
        const labelElement = document.querySelector('.control-label');
        if (activeOverlays.length > 0) {
            labelElement.textContent = `Ativos: ${activeOverlays.join(' | ')}`;
            labelElement.classList.add('active');
        } else {
            labelElement.textContent = 'Clique nas imagens para ativar';
            labelElement.classList.remove('active');
        }
    }
    
    hasActiveOverlays() { return Array.from(this.overlayStates.values()).some(isActive => isActive); }
    getConfiguration() {
        return {
            addSpeedoOverlay: this.overlayStates.get('speedometer'), addTrackOverlay: this.overlayStates.get('map'), addStatsOverlay: this.overlayStates.get('stats'),
            speedoPosition: this.overlayPositions.get('speedometer') || null, trackPosition: this.overlayPositions.get('map') || null, statsPosition: this.overlayPositions.get('stats') || null
        };
    }
}