// js/file-handlers.js - Manipulação de arquivos e upload

// Variáveis de estado dos arquivos
let gpxFile = null;
let videoFile = null;

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