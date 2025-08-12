export async function fetchSuggestion(context) {
    const { gpxFile, videoFile, interpolationSlider, notify, t } = context;

    if (!gpxFile || !videoFile) return null;
    
    notify.info(t('notification_suggestion'), t('analyzing_files'));
    
    const formData = new FormData();
    formData.append('gpxFile', gpxFile);
    formData.append('videoFile', videoFile);
    formData.append('interpolationLevel', interpolationSlider.value);
    
    try {
        const response = await fetch('/suggest', { method: 'POST', body: formData });
        const data = await response.json();
        
        if (!response.ok) {
            notify.error(t('notification_error'), t('suggestion_error', { message: data.message || 'Unknown error' }));
            return null;
        }
        
        if (data.interpolated_points && data.interpolated_points.length > 0) {
            notify.success(t('notification_suggestion'), t('high_precision_track_loaded'));
        }
        
        if (data.timestamp) {
            notify.success(t('notification_suggestion'), t('suggestion_applied'));
        }
        
        return data; // Retorna { interpolated_points, timestamp, latitude, longitude, display_timestamp }
        
    } catch (error) {
        console.error("Error calling /suggest endpoint:", error);
        notify.error(t('notification_error'), t('suggestion_comm_error'));
        return null;
    }
}

export function generateVideo(context) {
    const { gpxFile, videoFile, selectedSyncPoint, interpolationSlider, inlineOverlayManager, loader, notify, t, onFinish } = context;

    if (!gpxFile || !videoFile || !selectedSyncPoint) { 
        notify.error(t('notification_error'), t('error_missing_files'));
        return; 
    }
    
    if (!inlineOverlayManager.hasActiveOverlays()) {
        notify.error(t('notification_error'), t('error_missing_overlay'));
        return;
    }

    const formData = new FormData();
    formData.append('gpxFile', gpxFile);
    formData.append('videoFile', videoFile);
    formData.append('syncTimestamp', selectedSyncPoint.time.toISOString());
    formData.append('lang', context.lang);
    formData.append('interpolationLevel', interpolationSlider.value);

    const overlayConfig = inlineOverlayManager.getConfiguration();
    for (const key in overlayConfig) {
        if (overlayConfig[key] !== null) {
            formData.append(key, overlayConfig[key]);
        }
    }

    const xhr = new XMLHttpRequest();
    xhr.open('POST', '/process', true);
    
    xhr.upload.onprogress = (event) => { 
        if (event.lengthComputable) { 
            loader.setProgress(Math.round((event.loaded / event.total) * 15)); // Upload é 15% do total
        } 
    };
    
    xhr.onload = () => {
        loader.hide();
        const result = JSON.parse(xhr.responseText);
        
        if (xhr.status >= 200 && xhr.status < 300) {
            notify.success(t('notification_success'), t('success_message'));
            onFinish(result, null);
        } else {
            notify.error(t('notification_error'), t('server_error', { message: result.message || 'Unknown error' }));
            onFinish(null, result);
        }
    };
    
    xhr.onerror = () => { 
        loader.hide();
        notify.error(t('notification_error'), t('network_error'));
        onFinish(null, { message: 'Network Error' });
    };
    
    xhr.send(formData);
}