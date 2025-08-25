// js/strava-manager.js - Gerenciamento da integração com Strava

class StravaManager {
    constructor() {
        this.sessionId = localStorage.getItem('strava_session_id');
        this.isAuthenticated = false;
        this.activities = [];
        this.selectedActivity = null;
        
        this.init();
    }
    
    init() {
        this.createStravaInterface();
        this.addEventListeners();
        
        // Verificar se já temos uma sessão válida
        if (this.sessionId) {
            this.checkAuthStatus();
        }
        
        // Verificar se estamos voltando do callback do Strava
        this.handleCallback();
    }
    
    createStravaInterface() {
        const filesContainer = document.querySelector('.files-container');
        if (!filesContainer) return;
        
        // Adicionar seção do Strava antes dos uploads manuais
        const stravaSection = document.createElement('div');
        stravaSection.className = 'file-group strava-integration';
        stravaSection.innerHTML = `
            <label class="file-group-label">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="#fc4c02">
                    <path d="M15.387 17.064l-4.834-12.351c-.348-.854-1.556-.854-1.904 0L4.815 17.064c-.348.854.389 1.791 1.301 1.533l3.862-1.096c.681-.194 1.408-.194 2.089 0l3.862 1.096c.912.258 1.649-.679 1.301-1.533z"/>
                </svg>
                <span data-i18n="strava_integration">🔗 Integração Strava</span>
            </label>
            <div id="strava-content">
                <div id="strava-not-connected" class="strava-state">
                    <button id="strava-connect-btn" class="strava-btn primary">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M15.387 17.064l-4.834-12.351c-.348-.854-1.556-.854-1.904 0L4.815 17.064c-.348.854.389 1.791 1.301 1.533l3.862-1.096c.681-.194 1.408-.194 2.089 0l3.862 1.096c.912.258 1.649-.679 1.301-1.533z"/>
                        </svg>
                        <span data-i18n="strava_connect">Conectar ao Strava</span>
                    </button>
                    <div class="strava-info">
                        <p data-i18n="strava_connect_desc">Conecte-se ao Strava para importar suas atividades diretamente.</p>
                    </div>
                </div>
                
                <div id="strava-connected" class="strava-state hidden">
                    <div class="strava-controls">
                        <button id="strava-activities-btn" class="strava-btn secondary">
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M4 6h16M4 12h16M4 18h16"/>
                            </svg>
                            <span data-i18n="strava_load_activities">Carregar Atividades</span>
                        </button>
                        <button id="strava-disconnect-btn" class="strava-btn danger small">
                            <span data-i18n="strava_disconnect">Desconectar</span>
                        </button>
                    </div>
                    
                    <div id="strava-activities-list" class="strava-activities hidden">
                        <div class="activities-header">
                            <h4 data-i18n="strava_recent_activities">Atividades Recentes</h4>
                            <div class="activities-controls">
                                <select id="strava-format-select">
                                    <option value="fit" data-i18n="format_fit">FIT (Recomendado)</option>
                                    <option value="tcx" data-i18n="format_tcx">TCX</option>
                                    <option value="gpx" data-i18n="format_gpx">GPX</option>
                                </select>
                            </div>
                        </div>
                        <div id="activities-container" class="activities-container">
                            <div class="loading-activities">
                                <div class="loading-spinner-small"></div>
                                <span data-i18n="strava_loading_activities">Carregando atividades...</span>
                            </div>
                        </div>
                    </div>
                </div>
                
                <div id="strava-error" class="strava-state strava-error hidden">
                    <div class="error-content">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="10"/>
                            <line x1="15" y1="9" x2="9" y2="15"/>
                            <line x1="9" y1="9" x2="15" y2="15"/>
                        </svg>
                        <div>
                            <strong data-i18n="strava_error_title">Erro na integração Strava</strong>
                            <p id="strava-error-message"></p>
                        </div>
                    </div>
                    <button id="strava-retry-btn" class="strava-btn primary small">
                        <span data-i18n="strava_retry">Tentar Novamente</span>
                    </button>
                </div>
            </div>
        `;
        
        filesContainer.insertBefore(stravaSection, filesContainer.firstChild);
    }
    
    addEventListeners() {
        const connectBtn = document.getElementById('strava-connect-btn');
        const disconnectBtn = document.getElementById('strava-disconnect-btn');
        const activitiesBtn = document.getElementById('strava-activities-btn');
        const retryBtn = document.getElementById('strava-retry-btn');
        
        if (connectBtn) {
            connectBtn.addEventListener('click', () => this.initiateAuth());
        }
        
        if (disconnectBtn) {
            disconnectBtn.addEventListener('click', () => this.disconnect());
        }
        
        if (activitiesBtn) {
            activitiesBtn.addEventListener('click', () => this.loadActivities());
        }
        
        if (retryBtn) {
            retryBtn.addEventListener('click', () => this.clearError());
        }
    }
    
    showState(state) {
        document.querySelectorAll('.strava-state').forEach(el => el.classList.add('hidden'));
        document.getElementById(`strava-${state}`).classList.remove('hidden');
    }
    
    showError(message) {
        document.getElementById('strava-error-message').textContent = message;
        this.showState('error');
    }
    
    clearError() {
        this.showState(this.isAuthenticated ? 'connected' : 'not-connected');
    }
    
    async initiateAuth() {
        try {
            const response = await fetch('/strava/auth');
            const data = await response.json();
            
            if (data.auth_url) {
                // Abrir nova janela para autenticação
                const authWindow = window.open(
                    data.auth_url, 
                    'strava_auth', 
                    'width=600,height=800,scrollbars=yes,resizable=yes'
                );
                
                // Monitorar quando a janela é fechada
                const checkClosed = setInterval(() => {
                    if (authWindow.closed) {
                        clearInterval(checkClosed);
                        // Verificar se a autenticação foi bem-sucedida
                        setTimeout(() => this.checkAuthStatus(), 1000);
                    }
                }, 1000);
                
            } else {
                this.showError(data.error || t('strava_config_error'));
            }
        } catch (error) {
            console.error('Erro ao iniciar autenticação Strava:', error);
            this.showError(t('strava_auth_error'));
        }
    }
    
    async checkAuthStatus() {
        if (!this.sessionId) return;
        
        try {
            const response = await fetch(`/strava/status?session_id=${this.sessionId}`);
            const data = await response.json();
            
            if (data.authenticated && data.token_valid) {
                this.isAuthenticated = true;
                this.showState('connected');
                notify.success(t('notification_strava_connected'), t('strava_connected_success'));
            } else {
                this.isAuthenticated = false;
                this.sessionId = null;
                localStorage.removeItem('strava_session_id');
                this.showState('not-connected');
            }
        } catch (error) {
            console.error('Erro ao verificar status Strava:', error);
            this.isAuthenticated = false;
            this.showState('not-connected');
        }
    }
    
    handleCallback() {
        const urlParams = new URLSearchParams(window.location.search);
        const code = urlParams.get('code');
        const error = urlParams.get('error');
        
        if (error) {
            this.showError(t('strava_auth_denied'));
            // Limpar parâmetros da URL
            window.history.replaceState({}, document.title, window.location.pathname);
            return;
        }
        
        if (code) {
            // Processar callback de autenticação
            this.processCallback(code);
            // Limpar parâmetros da URL
            window.history.replaceState({}, document.title, window.location.pathname);
        }
    }
    
    async processCallback(code) {
        try {
            const response = await fetch(`/strava/callback?code=${code}`);
            const data = await response.json();
            
            if (data.success) {
                // Extrair session ID da mensagem (melhorar isso em produção)
                const sessionMatch = data.message.match(/Session ID: (.+)$/);
                if (sessionMatch) {
                    this.sessionId = sessionMatch[1];
                    localStorage.setItem('strava_session_id', this.sessionId);
                    this.isAuthenticated = true;
                    this.showState('connected');
                    notify.success(t('notification_strava_connected'), t('strava_connected_success'));
                }
            } else {
                this.showError(data.message);
            }
        } catch (error) {
            console.error('Erro no callback Strava:', error);
            this.showError(t('strava_callback_error'));
        }
    }
    
    async loadActivities() {
        if (!this.sessionId) {
            this.showError(t('strava_not_authenticated'));
            return;
        }
        
        const activitiesList = document.getElementById('strava-activities-list');
        const container = document.getElementById('activities-container');
        
        activitiesList.classList.remove('hidden');
        container.innerHTML = `
            <div class="loading-activities">
                <div class="loading-spinner-small"></div>
                <span data-i18n="strava_loading_activities">Carregando atividades...</span>
            </div>
        `;
        
        try {
            const response = await fetch(`/strava/activities?session_id=${this.sessionId}&per_page=20`);
            const data = await response.json();
            
            if (data.success && data.activities) {
                this.activities = data.activities;
                this.renderActivities();
            } else {
                this.showError(data.error || t('strava_activities_error'));
            }
        } catch (error) {
            console.error('Erro ao carregar atividades:', error);
            this.showError(t('strava_network_error'));
        }
    }
    
    renderActivities() {
        const container = document.getElementById('activities-container');
        
        if (this.activities.length === 0) {
            container.innerHTML = `
                <div class="no-activities">
                    <p data-i18n="strava_no_activities">Nenhuma atividade encontrada.</p>
                </div>
            `;
            return;
        }
        
        container.innerHTML = this.activities.map(activity => `
            <div class="activity-item" data-activity-id="${activity.id}">
                <div class="activity-header">
                    <div class="activity-icon">
                        ${this.getActivityIcon(activity.sport_type)}
                    </div>
                    <div class="activity-info">
                        <h5 class="activity-name">${this.escapeHtml(activity.name)}</h5>
                        <div class="activity-meta">
                            <span class="activity-sport">${activity.sport_type}</span>
                            <span class="activity-date">${this.formatDate(activity.start_date_local)}</span>
                        </div>
                    </div>
                </div>
                <div class="activity-stats">
                    <div class="stat">
                        <span class="stat-label" data-i18n="distance">Distância:</span>
                        <span class="stat-value">${(activity.distance / 1000).toFixed(1)} km</span>
                    </div>
                    <div class="stat">
                        <span class="stat-label" data-i18n="duration">Duração:</span>
                        <span class="stat-value">${this.formatDuration(activity.moving_time)}</span>
                    </div>
                    <div class="stat">
                        <span class="stat-label" data-i18n="elevation">Elevação:</span>
                        <span class="stat-value">${activity.total_elevation_gain.toFixed(0)} m</span>
                    </div>
                </div>
                <div class="activity-actions">
                    <button class="activity-select-btn" data-activity-id="${activity.id}">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                            <polyline points="7,10 12,15 17,10"/>
                            <line x1="12" y1="15" x2="12" y2="3"/>
                        </svg>
                        <span data-i18n="strava_select_activity">Selecionar</span>
                    </button>
                </div>
            </div>
        `).join('');
        
        // Adicionar event listeners aos botões
        container.querySelectorAll('.activity-select-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const activityId = e.currentTarget.dataset.activityId;
                this.selectActivity(activityId);
            });
        });
    }
    
    async selectActivity(activityId) {
        const format = document.getElementById('strava-format-select').value;
        const btn = document.querySelector(`[data-activity-id="${activityId}"] .activity-select-btn`);
        const originalText = btn.innerHTML;
        
        // Mostrar loading no botão
        btn.innerHTML = `
            <div class="loading-spinner-small"></div>
            <span data-i18n="strava_downloading">Baixando...</span>
        `;
        btn.disabled = true;
        
        try {
            const response = await fetch(`/strava/download/${activityId}?session_id=${this.sessionId}`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ format })
            });
            
            const data = await response.json();
            
            if (response.ok) {
                this.selectedActivity = this.activities.find(a => a.id == activityId);
                
                // Simular o carregamento de arquivo GPX para o sistema existente
                this.simulateGpxUpload(data, format);
                
                notify.success(t('notification_strava_activity'), 
                    t('strava_activity_imported', { name: this.selectedActivity.name }));
                
                // Marcar atividade como selecionada
                document.querySelectorAll('.activity-item').forEach(item => {
                    item.classList.remove('selected');
                });
                document.querySelector(`[data-activity-id="${activityId}"]`).classList.add('selected');
                
            } else {
                notify.error(t('notification_error'), data.message || t('strava_download_error'));
            }
        } catch (error) {
            console.error('Erro ao baixar atividade:', error);
            notify.error(t('notification_error'), t('strava_network_error'));
        } finally {
            // Restaurar botão
            btn.innerHTML = originalText;
            btn.disabled = false;
        }
    }
    
    simulateGpxUpload(data, format) {
        // Simular que um arquivo foi carregado
        gpxFile = { 
            name: `strava_activity_${format.toUpperCase()}.${format}`,
            strava_data: data
        };
        
        const gpxInfo = document.getElementById('gpx-info');
        const videoInput = document.getElementById('video-file');
        const videoInfo = document.getElementById('video-info');
        const trackInfoDiv = document.getElementById('track-info');
        
        gpxInfo.textContent = `${format.toUpperCase()}: ${this.selectedActivity.name}`;
        videoInput.disabled = false;
        videoInfo.textContent = t('can_select_video');
        
        // Mostrar informações da atividade Strava
        if (data.extra_data) {
            let infoHtml = `<h4>📈 ${t('strava_activity_imported_title')}</h4>`;
            infoHtml += `<p><strong>${this.selectedActivity.name}</strong></p>`;
            infoHtml += `<p>${t('strava_sport_detected', { sport: this.selectedActivity.sport_type })}</p>`;
            
            const stats = [];
            if (data.extra_data.total_calories > 0) {
                stats.push(t('tcx_calories', { calories: Math.round(data.extra_data.total_calories) }));
            }
            if (data.extra_data.average_heart_rate) {
                stats.push(t('tcx_heart_rate', { 
                    avg: Math.round(data.extra_data.average_heart_rate), 
                    max: Math.round(data.extra_data.max_heart_rate) 
                }));
            }
            
            if (stats.length > 0) {
                infoHtml += '<ul>';
                stats.forEach(stat => {
                    infoHtml += `<li>${stat}</li>`;
                });
                infoHtml += '</ul>';
            }
            
            trackInfoDiv.innerHTML = infoHtml;
            trackInfoDiv.style.display = 'block';
        }
        
        // Simular os dados para o sistema existente
        if (data.interpolated_points) {
            gpxDataPoints = data.interpolated_points.map(p => ({ 
                ...p, 
                time: p.time ? new Date(p.time) : null 
            }));
            displayTrack(gpxDataPoints);
        }
        
        if (data.latitude && data.longitude) {
            const suggestedPoint = {
                lat: data.latitude,
                lon: data.longitude,
                time: data.timestamp ? new Date(data.timestamp) : new Date(),
                displayTime: data.display_timestamp
            };
            selectSyncPoint(suggestedPoint, true);
        }
        
        validateGenerateButton();
        checkAndShowMapSection();
    }
    
    disconnect() {
        this.sessionId = null;
        this.isAuthenticated = false;
        this.activities = [];
        this.selectedActivity = null;
        
        localStorage.removeItem('strava_session_id');
        this.showState('not-connected');
        
        // Limpar dados do sistema
        document.getElementById('strava-activities-list').classList.add('hidden');
        
        notify.info(t('notification_strava_disconnected'), t('strava_disconnected'));
    }
    
    // Utility methods
    getActivityIcon(sportType) {
        const icons = {
            'Ride': '🚴',
            'Run': '🏃',
            'Walk': '🚶',
            'Hike': '🥾',
            'Swim': '🏊',
            'default': '🏃'
        };
        return icons[sportType] || icons.default;
    }
    
    formatDate(dateStr) {
        const date = new Date(dateStr);
        return date.toLocaleDateString(currentLang, { 
            day: 'numeric', 
            month: 'short',
            year: 'numeric'
        });
    }
    
    formatDuration(seconds) {
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        
        if (hours > 0) {
            return `${hours}h ${minutes}min`;
        }
        return `${minutes}min`;
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Instância global
let stravaManager;