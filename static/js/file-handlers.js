// js/file-handlers.js - Manipulação de arquivos e upload

// Variáveis de estado dos arquivos
let gpxFile = null;
let videoFile = null;

function handleGpxUpload(event) { 
    gpxFile = event.target.files[0]; 
    if (!gpxFile) return; 
    
    // NOVO: Detectar tipo de arquivo
    const fileExt = gpxFile.name.split('.').pop().toLowerCase();
    const fileTypeText = fileExt === 'tcx' ? 'TCX' : 'GPX';
    
    gpxInfo.textContent = `${fileTypeText}: ${gpxFile.name}`; 
    notify.success(t('notification_gpx_loaded'), `${fileTypeText} - ${t('gpx_loaded')}`);
    
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
            
            // NOVO: Mostrar informações extras do TCX se disponível
            if (data.file_type === 'TCX' && data.extra_data) {
                showTcxExtraInfo(data.extra_data, data.sport_type);
            }
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

// NOVA: Função para exibir informações extras do TCX
function showTcxExtraInfo(extraData, sportType) {
    const trackInfoDiv = document.getElementById('track-info');
    
    let infoHtml = `<h4>📊 ${t('tcx_extra_data_loaded')}</h4>`;
    
    if (sportType) {
        infoHtml += `<p><strong>${t('tcx_sport_detected', { sport: sportType })}</strong></p>`;
    }
    
    const stats = [];
    
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
    
    if (stats.length > 0) {
        infoHtml += '<ul>';
        stats.forEach(stat => {
            infoHtml += `<li>${stat}</li>`;
        });
        infoHtml += '</ul>';
    }
    
    trackInfoDiv.innerHTML = infoHtml;
    trackInfoDiv.style.display = 'block';
    
    notify.info(t('notification_suggestion'), t('tcx_extra_data_loaded'));
}