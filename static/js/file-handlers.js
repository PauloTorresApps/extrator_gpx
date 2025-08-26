// js/file-handlers.js - Manipulação de arquivos CORRIGIDA

// Variáveis de estado dos arquivos
let gpxFile = null;
let videoFile = null;

function handleGpxUpload(event) { 
    const file = event.target.files[0]; 
    if (!file) {
        // Reset se não há arquivo
        gpxFile = null;
        gpxInfo.textContent = t('no_gpx_selected');
        videoInput.disabled = true;
        videoInfo.textContent = t('select_gpx_first');
        hideMapAndPositionSections();
        validateGenerateButton();
        return;
    }
    
    // Verificar extensão do arquivo
    const fileName = file.name.toLowerCase();
    const validExtensions = ['.gpx', '.tcx', '.fit'];
    const isValidFile = validExtensions.some(ext => fileName.endsWith(ext));
    
    if (!isValidFile) {
        notify.error(t('notification_error'), 
            `Tipo de arquivo não suportado. Use: ${validExtensions.join(', ')}`);
        event.target.value = ''; // Limpar input
        return;
    }
    
    // Verificar tamanho do arquivo (máximo 50MB)
    const maxSize = 50 * 1024 * 1024; // 50MB em bytes
    if (file.size > maxSize) {
        notify.error(t('notification_error'), 
            `Arquivo muito grande. Tamanho máximo: 50MB`);
        event.target.value = ''; // Limpar input
        return;
    }
    
    gpxFile = file;
    
    // Detectar tipo de arquivo
    const fileExt = fileName.split('.').pop().toUpperCase();
    const fileTypeEmoji = {
        'GPX': '🗺️',
        'TCX': '📊', 
        'FIT': '⌚'
    };
    
    gpxInfo.textContent = `${fileTypeEmoji[fileExt] || '📁'} ${fileExt}: ${file.name}`;
    gpxInfo.title = `Tamanho: ${formatFileSize(file.size)} | Tipo: ${fileExt}`;
    
    notify.success(t('notification_gpx_loaded'), 
        `${fileExt} carregado: ${file.name}`);
    
    // Habilitar seleção de vídeo
    videoInput.disabled = false;
    videoInfo.textContent = t('can_select_video');
    
    // Resetar dados anteriores
    resetTrackData();
    
    validateGenerateButton();
    
    // Buscar sugestão automaticamente
    fetchAndApplySuggestion();
}

function handleVideoUpload(event) { 
    const file = event.target.files[0]; 
    if (!file) {
        // Reset se não há arquivo
        videoFile = null;
        videoInfo.textContent = t('can_select_video');
        hideMapAndPositionSections();
        validateGenerateButton();
        return;
    }
    
    // Verificar se é um arquivo de vídeo
    const validVideoTypes = [
        'video/mp4', 'video/avi', 'video/mov', 'video/mkv', 
        'video/wmv', 'video/flv', 'video/webm', 'video/m4v'
    ];
    
    if (!validVideoTypes.includes(file.type) && !file.name.match(/\.(mp4|avi|mov|mkv|wmv|flv|webm|m4v)$/i)) {
        notify.error(t('notification_error'), 
            'Formato de vídeo não suportado. Use: MP4, AVI, MOV, MKV, etc.');
        event.target.value = ''; // Limpar input
        return;
    }
    
    // Verificar tamanho do arquivo (máximo 2GB)
    const maxSize = 2 * 1024 * 1024 * 1024; // 2GB em bytes
    if (file.size > maxSize) {
        notify.error(t('notification_error'), 
            `Vídeo muito grande. Tamanho máximo: 2GB`);
        event.target.value = ''; // Limpar input
        return;
    }
    
    videoFile = file;
    
    videoInfo.textContent = `🎬 ${file.name}`;
    videoInfo.title = `Tamanho: ${formatFileSize(file.size)} | Duração: Verificando...`;
    
    notify.success(t('notification_video_loaded'), file.name);
    
    // Mostrar seções do mapa e posicionamento
    checkAndShowMapSection();
    validateGenerateButton();
    
    // Re-executar sugestão se já temos um arquivo GPX
    if (gpxFile) {
        fetchAndApplySuggestion();
    }
    
    // Tentar obter duração do vídeo (se possível no browser)
    tryGetVideoDuration(file);
}

function checkAndShowMapSection() { 
    if (gpxFile && videoFile) { 
        mapSection.style.display = 'block';
        syncPointInfo.style.display = 'block';
        syncPointInfo.textContent = t('map_click_prompt');
        
        // CORREÇÃO: Invalidar tamanho do mapa após mostrar e aguardar
        setTimeout(() => {
            if (map) {
                console.log('🗺️ Invalidando tamanho do mapa...');
                map.invalidateSize();
                
                // Se já temos pontos, reajustar o zoom
                if (gpxDataPoints && gpxDataPoints.length > 0) {
                    console.log('🗺️ Reajustando zoom para pontos existentes...');
                    displayTrack(gpxDataPoints);
                }
            }
        }, 300); // Tempo suficiente para a transição CSS
    } 
}

async function fetchAndApplySuggestion() {
    if (!gpxFile || !videoFile) return;
    
    console.log('🔍 INÍCIO - Buscando sugestão de sincronização...');
    
    // Mostrar indicador de carregamento
    const originalGpxInfo = gpxInfo.textContent;
    gpxInfo.textContent = `${originalGpxInfo} (Analisando...)`;
    
    notify.info(t('notification_suggestion'), t('analyzing_files'));
    
    const formData = new FormData();
    formData.append('gpxFile', gpxFile);
    formData.append('videoFile', videoFile);
    formData.append('interpolationLevel', interpolationSlider?.value || '1');
    
    try {
        console.log('📡 Enviando requisição para /suggest...');
        
        const response = await fetch('/suggest', { 
            method: 'POST', 
            body: formData 
        });
        
        console.log('📡 Resposta recebida:', response.status, response.statusText);
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const data = await response.json();
        console.log('📊 Dados recebidos:', data);
        
        // Restaurar texto original
        gpxInfo.textContent = originalGpxInfo;
        
        // CORREÇÃO: Processar pontos interpolados
        if (data.interpolated_points && data.interpolated_points.length > 0) {
            console.log(`✅ ${data.interpolated_points.length} pontos interpolados recebidos`);
            
            notify.success(t('notification_suggestion'), 
                `Trilha carregada: ${data.interpolated_points.length} pontos`);
            
            // CORREÇÃO: Converter timestamps e validar dados
            const processedPoints = data.interpolated_points
                .map((p, index) => {
                    // Validar estrutura do ponto
                    if (!p || (typeof p.lat === 'undefined' && typeof p.latitude === 'undefined')) {
                        console.warn(`⚠️ Ponto ${index} sem coordenadas válidas:`, p);
                        return null;
                    }
                    
                    // Normalizar estrutura
                    const processedPoint = {
                        lat: p.lat || p.latitude,
                        lon: p.lon || p.longitude,
                        time: p.time ? new Date(p.time) : null,
                        heart_rate: p.heart_rate,
                        cadence: p.cadence,
                        speed: p.speed,
                        original_index: index
                    };
                    
                    // Validar coordenadas
                    const lat = parseFloat(processedPoint.lat);
                    const lon = parseFloat(processedPoint.lon);
                    
                    if (isNaN(lat) || isNaN(lon) || lat < -90 || lat > 90 || lon < -180 || lon > 180) {
                        console.warn(`⚠️ Ponto ${index} com coordenadas inválidas:`, {lat, lon});
                        return null;
                    }
                    
                    processedPoint.lat = lat;
                    processedPoint.lon = lon;
                    
                    return processedPoint;
                })
                .filter(p => p !== null); // Remover pontos inválidos
            
            console.log(`✅ ${processedPoints.length} pontos válidos após processamento`);
            
            if (processedPoints.length > 0) {
                // Armazenar pontos globalmente
                gpxDataPoints = processedPoints;
                
                // DEBUG: Mostrar amostra dos pontos
                console.log('🔍 Amostra dos primeiros 3 pontos processados:');
                processedPoints.slice(0, 3).forEach((p, i) => {
                    console.log(`Ponto ${i}:`, {
                        lat: p.lat,
                        lon: p.lon,
                        time: p.time,
                        speed: p.speed
                    });
                });
                
                // Exibir trilha no mapa
                displayTrack(processedPoints);
            } else {
                console.error('❌ Nenhum ponto válido após processamento');
                notify.error(t('notification_error'), 'Nenhuma coordenada válida encontrada nos dados');
            }
            
            // Mostrar informações extras do arquivo
            if (data.file_type && (data.file_type === 'TCX' || data.file_type === 'FIT') && data.extra_data) {
                showExtraTrackInfo(data.extra_data, data.sport_type, data.file_type);
            }
        } else {
            console.warn('⚠️ Nenhum ponto interpolado recebido');
            notify.warning(t('notification_suggestion'), 'Nenhum ponto de trilha válido encontrado');
        }
        
        // CORREÇÃO: Aplicar sugestão de ponto de sincronização
        if (data.timestamp && data.latitude && data.longitude) {
            console.log('🎯 Aplicando sugestão de sincronização...');
            
            const suggestedPoint = { 
                lat: parseFloat(data.latitude), 
                lon: parseFloat(data.longitude), 
                time: new Date(data.timestamp), 
                displayTime: data.display_timestamp 
            };
            
            // Validar coordenadas sugeridas
            if (!isNaN(suggestedPoint.lat) && !isNaN(suggestedPoint.lon) &&
                suggestedPoint.lat >= -90 && suggestedPoint.lat <= 90 &&
                suggestedPoint.lon >= -180 && suggestedPoint.lon <= 180) {
                
                console.log('✅ Ponto de sincronização válido:', suggestedPoint);
                selectSyncPoint(suggestedPoint, true);
                notify.success(t('notification_suggestion'), 
                    `Ponto de sincronização sugerido: ${data.display_timestamp || 'horário válido'}`);
            } else {
                console.error('❌ Coordenadas de sincronização inválidas:', suggestedPoint);
                notify.warning(t('notification_suggestion'), 
                    'Ponto de sincronização inválido. Selecione manualmente no mapa.');
            }
        } else if (gpxDataPoints && gpxDataPoints.length > 0) {
            console.log('⚠️ Sem sugestão específica, mas trilha carregada');
            notify.info(t('notification_suggestion'), 
                'Trilha carregada. Selecione manualmente o ponto de sincronização no mapa.');
        }
        
        console.log('✅ SUCESSO - Sugestão processada com sucesso');
        
    } catch (error) {
        console.error('❌ ERRO - Falha ao buscar sugestão:', error);
        
        // Restaurar texto original
        gpxInfo.textContent = originalGpxInfo;
        
        notify.error(t('notification_error'), 
            `Erro ao analisar arquivos: ${error.message}`);
        
        // Mesmo com erro na sugestão, permitir continuar manualmente
        if (gpxFile && videoFile) {
            notify.info(t('notification_suggestion'), 
                'Selecione manualmente um ponto no mapa para sincronização.');
        }
    }
    
    console.log('🔍 FIM - Processamento de sugestão finalizado');
}

function showExtraTrackInfo(extraData, sportType, fileType) {
    const trackInfoDiv = document.getElementById('track-info');
    if (!trackInfoDiv) return;
    
    console.log('📊 Mostrando informações extras:', { extraData, sportType, fileType });
    
    let infoHtml = `<h4>📊 ${fileType} - ${t('tcx_extra_data_loaded')}</h4>`;
    
    if (sportType) {
        infoHtml += `<p><strong>${t('tcx_sport_detected', { sport: sportType })}</strong></p>`;
    }
    
    const stats = [];
    
    if (extraData.total_distance_meters > 0) {
        stats.push(`📏 Distância: ${(extraData.total_distance_meters / 1000).toFixed(2)} km`);
    }
    
    if (extraData.total_time_seconds > 0) {
        const hours = Math.floor(extraData.total_time_seconds / 3600);
        const minutes = Math.floor((extraData.total_time_seconds % 3600) / 60);
        stats.push(`⏱️ Duração: ${hours > 0 ? hours + 'h ' : ''}${minutes}min`);
    }
    
    if (extraData.total_calories > 0) {
        stats.push(t('tcx_calories', { calories: Math.round(extraData.total_calories) }));
    }
    
    if (extraData.average_heart_rate && extraData.max_heart_rate) {
        stats.push(t('tcx_heart_rate', { 
            avg: Math.round(extraData.average_heart_rate), 
            max: Math.round(extraData.max_heart_rate) 
        }));
    }
    
    if (extraData.average_cadence && extraData.max_cadence) {
        stats.push(t('tcx_cadence', { 
            avg: Math.round(extraData.average_cadence), 
            max: Math.round(extraData.max_cadence) 
        }));
    }
    
    if (extraData.max_speed > 0) {
        stats.push(`🏃 Velocidade máx: ${(extraData.max_speed * 3.6).toFixed(1)} km/h`);
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
    
    notify.info(t('notification_suggestion'), 
        `${fileType} com telemetria completa detectado!`);
}

function resetTrackData() {
    console.log('🧹 Resetando dados da trilha...');
    
    // Limpar dados da trilha anterior
    gpxDataPoints = [];
    selectedSyncPoint = null;
    
    // Limpar mapa
    if (typeof clearMap === 'function') {
        clearMap();
    }
    
    // Ocultar seções
    hideMapAndPositionSections();
    
    // Limpar info track
    const trackInfoDiv = document.getElementById('track-info');
    if (trackInfoDiv) {
        trackInfoDiv.style.display = 'none';
        trackInfoDiv.innerHTML = '';
    }
    
    console.log('✅ Dados da trilha resetados');
}

function hideMapAndPositionSections() {
    if (mapSection) mapSection.style.display = 'none';
    if (positionSection) positionSection.style.display = 'none';
    if (syncPointInfo) syncPointInfo.style.display = 'none';
}

function tryGetVideoDuration(file) {
    // Tentar obter duração do vídeo usando HTML5 Video API
    const video = document.createElement('video');
    video.preload = 'metadata';
    
    video.onloadedmetadata = function() {
        const duration = Math.round(video.duration);
        const minutes = Math.floor(duration / 60);
        const seconds = duration % 60;
        
        videoInfo.title = videoInfo.title.replace('Duração: Verificando...', 
            `Duração: ${minutes}:${seconds.toString().padStart(2, '0')}`);
        
        URL.revokeObjectURL(video.src);
    };
    
    video.onerror = function() {
        // Se falhar, não faz nada - informação opcional
        URL.revokeObjectURL(video.src);
    };
    
    video.src = URL.createObjectURL(file);
}

function formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// Função para validar se os arquivos ainda estão válidos
function validateFiles() {
    let valid = true;
    
    if (gpxFile && !gpxFile.name) {
        // Arquivo GPX foi corrompido/perdido
        gpxFile = null;
        gpxInfo.textContent = t('no_gpx_selected');
        valid = false;
    }
    
    if (videoFile && !videoFile.name) {
        // Arquivo de vídeo foi corrompido/perdido  
        videoFile = null;
        videoInfo.textContent = gpxFile ? t('can_select_video') : t('select_gpx_first');
        valid = false;
    }
    
    if (!valid) {
        resetTrackData();
        validateGenerateButton();
    }
    
    return valid;
}

// Função para limpar tudo
function resetAllFiles() {
    console.log('🧹 Resetando todos os arquivos...');
    
    gpxFile = null;
    videoFile = null;
    
    // Limpar inputs
    const gpxInput = document.getElementById('gpx-file');
    const videoInput = document.getElementById('video-file');
    
    if (gpxInput) gpxInput.value = '';
    if (videoInput) videoInput.value = '';
    
    // Resetar UI
    gpxInfo.textContent = t('no_gpx_selected');
    videoInfo.textContent = t('select_gpx_first');
    videoInput.disabled = true;
    
    resetTrackData();
    validateGenerateButton();
    
    console.log('✅ Todos os arquivos resetados');
}

// NOVO: Função de debug para testar com arquivo sintético
function createSyntheticGpxFile() {
    const gpxContent = `<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="test">
    <trk>
        <trkseg>
            <trkpt lat="-15.7939" lon="-47.8828">
                <time>2023-01-01T10:00:00Z</time>
            </trkpt>
            <trkpt lat="-15.7949" lon="-47.8838">
                <time>2023-01-01T10:00:30Z</time>
            </trkpt>
            <trkpt lat="-15.7959" lon="-47.8848">
                <time>2023-01-01T10:01:00Z</time>
            </trkpt>
        </trkseg>
    </trk>
</gpx>`;
    
    const blob = new Blob([gpxContent], { type: 'application/gpx+xml' });
    const file = new File([blob], 'test.gpx', { type: 'application/gpx+xml' });
    
    // Simular seleção do arquivo
    gpxFile = file;
    gpxInfo.textContent = `🗺️ GPX: test.gpx (sintético)`;
    videoInput.disabled = false;
    videoInfo.textContent = t('can_select_video');
    
    notify.info('Debug', 'Arquivo GPX sintético criado para teste');
    
    // Simular dados processados
    const syntheticPoints = [
        { lat: -15.7939, lon: -47.8828, time: new Date('2023-01-01T10:00:00Z'), speed: 10 },
        { lat: -15.7949, lon: -47.8838, time: new Date('2023-01-01T10:00:30Z'), speed: 15 },
        { lat: -15.7959, lon: -47.8848, time: new Date('2023-01-01T10:01:00Z'), speed: 20 }
    ];
    
    gpxDataPoints = syntheticPoints;
    
    setTimeout(() => {
        displayTrack(syntheticPoints);
        selectSyncPoint(syntheticPoints[1], true);
    }, 500);
}

// Expor função para debug
window.createSyntheticGpxFile = createSyntheticGpxFile;
window.resetAllFiles = resetAllFiles;