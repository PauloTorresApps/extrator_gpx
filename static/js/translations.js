// js/translations.js - Sistema de traduções e internacionalização

const translations = {
    'en': {
        'main_title': '🎬 GPX/TCX Video Sync',
        'intro_text': 'Upload your track files (GPX or TCX), select a sync point on the map, and configure the overlays to generate your final video with telemetry.',
        'step1_title': 'Select Files', 'gpx_file_label': 'Track File (GPX/TCX)', 'choose_gpx': 'Choose GPX/TCX', 'no_gpx_selected': 'No file selected', 'video_file_label': 'Video File', 'choose_video': 'Choose Video', 'select_gpx_first': 'Select a track file first',
        'step2_title': 'Select Sync Point', 'map_click_prompt': '🎯 Click a point on the map to set it as the sync start.', 'step3_title': 'Positioning', 'speedo_label': '⏱️ Speedometer', 'map_label': '🗺️ Track Map', 'stats_label': '📊 Statistics',
        'generate_button': 'Confirm and Generate Video', 'download_link': '📥 Download Final Video', 'logs_title': '📋 Processing Logs:',
        'gpx_loaded': 'Track file loaded successfully', 'can_select_video': 'You can now select the video file', 'analyzing_files': 'Analyzing files to suggest sync point and track...', 'high_precision_track_loaded': 'High-precision track loaded from server.',
        'tcx_extra_data_loaded': 'TCX file detected! Extra telemetry data available: heart rate, cadence, and more.',
        'suggestion_applied': 'Automatic suggestion applied! You can adjust it on the map if needed.', 'suggestion_error': 'Could not get suggestion: {{message}}. Please select a point manually.', 'suggestion_comm_error': 'Communication error while getting suggestion. Please select a point manually.',
        'sync_point_selected': 'Point selected ({{type}}): {{time}} (UTC)', 'manual_type': 'manual', 'suggestion_type': 'suggestion', 'error_missing_files': 'Error: Please select both files and a sync point.',
        'uploading_files': 'Uploading files...', 'success_message': 'Success! Your video is ready.', 'server_error': 'Error: {{message}}', 'network_error': 'Network error while uploading files.',
        'settings_title': 'Advanced Settings', 'interpolation_label': 'Interpolation Precision Level', 'interpolation_desc': 'Lower value = more points = higher precision and slower processing.',
        'speedo_hint': 'Displays a speedometer with the current speed on the video.',
        'map_hint': 'Shows a mini-map with the traveled path and current position.',
        'stats_hint': 'Adds a panel with statistics like distance, time, and elevation.',
        'error_missing_overlay': 'Error: Please select at least one overlay (Speedometer, Map, or Statistics).',
        // TCX specific
        'tcx_sport_detected': 'Sport detected: {{sport}}',
        'tcx_calories': 'Total calories: {{calories}}',
        'tcx_heart_rate': 'Avg/Max heart rate: {{avg}}/{{max}} bpm',
        'tcx_cadence': 'Avg/Max cadence: {{avg}}/{{max}}',
        // Notificações
        'notification_gpx_loaded': 'Track Loaded',
        'notification_video_loaded': 'Video Loaded', 
        'notification_sync_selected': 'Sync Point Selected',
        'notification_processing': 'Processing Video',
        'notification_success': 'Video Ready',
        'notification_error': 'Error',
        'notification_suggestion': 'Auto Suggestion',
        // Loading
        'loading_title': 'Processing Video',
        'loading_message': 'Please wait while your video is being processed...',
        'step_upload': 'Uploading files',
        'step_analysis': 'Analyzing track data',
        'step_sync': 'Synchronizing with video',
        'step_overlays': 'Applying overlays',
        'step_render': 'Rendering final video',
        'loading_cancel': 'Cancel Processing',
        'download_tooltip': 'Download ready video',
        'settings_tooltip': 'Settings'
    },
    'pt-BR': {
        'main_title': '🎬 GPX/TCX Video Sync',
        'intro_text': 'Carregue os seus ficheiros de trilha (GPX ou TCX), selecione um ponto de sincronização no mapa e configure os overlays para gerar o seu vídeo final com telemetria.',
        'step1_title': 'Selecionar Ficheiros', 'gpx_file_label': 'Ficheiro de Trilha (GPX/TCX)', 'choose_gpx': 'Escolher GPX/TCX', 'no_gpx_selected': 'Nenhum ficheiro selecionado', 'video_file_label': 'Ficheiro de Vídeo', 'choose_video': 'Escolher Vídeo', 'select_gpx_first': 'Selecione um ficheiro de trilha primeiro',
        'step2_title': 'Selecionar Ponto de Sincronização', 'map_click_prompt': '🎯 Clique num ponto no mapa para o definir como o início da sincronização.', 'step3_title': 'Posicionamento', 'speedo_label': '⚙️ Velocímetro', 'map_label': '🗺️ Mapa do Trajeto', 'stats_label': '📊 Estatísticas',
        'generate_button': 'Confirmar e Gerar Vídeo', 'download_link': '📥 Descarregar Vídeo Final', 'logs_title': '📋 Logs do Processamento:',
        'gpx_loaded': 'Ficheiro de trilha carregado com sucesso', 'can_select_video': 'Agora pode selecionar o ficheiro de vídeo', 'analyzing_files': 'Analisando ficheiros para sugerir ponto e percurso...', 'high_precision_track_loaded': 'Percurso de alta precisão carregado do servidor.',
        'tcx_extra_data_loaded': 'Ficheiro TCX detectado! Dados extra de telemetria disponíveis: frequência cardíaca, cadência e mais.',
        'suggestion_applied': 'Sugestão automática aplicada! Pode ajustar no mapa se necessário.', 'suggestion_error': 'Não foi possível obter sugestão: {{message}}. Selecione um ponto manualmente.', 'suggestion_comm_error': 'Erro de comunicação ao obter sugestão. Selecione um ponto manualmente.',
        'sync_point_selected': 'Ponto selecionado ({{type}}): {{time}} (UTC)', 'manual_type': 'manual', 'suggestion_type': 'sugestão', 'error_missing_files': 'Erro: Por favor, selecione os dois ficheiros e um ponto de sincronização.',
        'uploading_files': 'A enviar ficheiros...', 'success_message': 'Sucesso! O seu vídeo está pronto.', 'server_error': 'Erro: {{message}}', 'network_error': 'Erro de rede ao enviar os ficheiros.',
        'settings_title': 'Configurações Avançadas', 'interpolation_label': 'Nível de Precisão da Interpolação', 'interpolation_desc': 'Menor valor = mais pontos = maior precisão e processamento mais lento.',
        'speedo_hint': 'Exibe um velocímetro com a velocidade atual no vídeo.',
        'map_hint': 'Mostra um mini-mapa com o trajeto percorrido e a posição atual.',
        'stats_hint': 'Adiciona um painel com estatísticas como distância, tempo e elevação.',
        'error_missing_overlay': 'Erro: Por favor, selecione pelo menos um overlay (Velocímetro, Mapa ou Estatísticas).',
        // TCX specific
        'tcx_sport_detected': 'Desporto detectado: {{sport}}',
        'tcx_calories': 'Calorias totais: {{calories}}',
        'tcx_heart_rate': 'Freq. cardíaca média/máx: {{avg}}/{{max}} bpm',
        'tcx_cadence': 'Cadência média/máx: {{avg}}/{{max}}',
        // Notificações
        'notification_gpx_loaded': 'Trilha Carregada',
        'notification_video_loaded': 'Vídeo Carregado',
        'notification_sync_selected': 'Ponto Selecionado',
        'notification_processing': 'Processando Vídeo', 
        'notification_success': 'Vídeo Pronto',
        'notification_error': 'Erro',
        'notification_suggestion': 'Sugestão Automática',
        // Loading
        'loading_title': 'Processando Vídeo',
        'loading_message': 'Por favor, aguarde enquanto seu vídeo é processado...',
        'step_upload': 'Enviando arquivos',
        'step_analysis': 'Analisando dados de trilha',
        'step_sync': 'Sincronizando com vídeo',
        'step_overlays': 'Aplicando overlays',
        'step_render': 'Renderizando vídeo final',
        'loading_cancel': 'Cancelar Processamento',
        'download_tooltip': 'Baixar vídeo pronto',
        'settings_tooltip': 'Configurações'
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
    
    // Atualizar tooltips
    if (downloadBtn) {
        downloadBtn.title = t('download_tooltip');
    }
    
    const settingsBtn = document.getElementById('settings-btn');
    if (settingsBtn) {
        settingsBtn.setAttribute('data-tooltip', t('settings_tooltip'));
    }
}