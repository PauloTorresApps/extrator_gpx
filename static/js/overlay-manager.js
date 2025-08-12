// js/overlay-manager.js - Sistema inline de overlays

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