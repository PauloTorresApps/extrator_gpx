// js/video-processor.js - Processamento e geração de vídeo

function validateGenerateButton() {
    const hasFiles = gpxFile && videoFile;
    const hasSyncPoint = selectedSyncPoint !== null;
    const hasActiveOverlay = inlineOverlayManager ? inlineOverlayManager.hasActiveOverlays() : false;
    
    const canGenerate = hasFiles && hasSyncPoint && hasActiveOverlay;
    
    generateBtn.disabled = !canGenerate;
    
    if (hasFiles && hasSyncPoint) {
        generateBtn.style.display = 'flex';
    } else {
        generateBtn.style.display = 'none';
    }
}

function handleGenerateWithInlineOverlays() {
    if (!gpxFile || !videoFile || !selectedSyncPoint) { 
        notify.error(t('notification_error'), t('error_missing_files'));
        return; 
    }
    
    if (!inlineOverlayManager.hasActiveOverlays()) {
        notify.error(t('notification_error'), t('error_missing_overlay'));
        return;
    }
    
    generateBtn.disabled = true;
    
    // Mostrar loading com simulação realística
    loader.show(() => {
        // Função de cancelamento (se o servidor suportar)
        notify.warning('Cancelamento', 'Processamento cancelado pelo usuário');
        generateBtn.disabled = false;
        validateGenerateButton();
    });
    
    // Iniciar simulação de progresso
    loader.simulateProgress();
    
    // Ocultar elementos da UI desnecessários durante o processamento
    logsContainer.style.display = 'none';
    downloadBtn.classList.remove('show'); // Ocultar botão de download
    
    const formData = new FormData();
    formData.append('gpxFile', gpxFile);
    formData.append('videoFile', videoFile);
    formData.append('syncTimestamp', selectedSyncPoint.time.toISOString());
    formData.append('lang', currentLang);
    formData.append('interpolationLevel', interpolationSlider.value);

    const overlayConfig = inlineOverlayManager.getConfiguration();
    
    formData.append('addSpeedoOverlay', overlayConfig.addSpeedoOverlay);
    if (overlayConfig.addSpeedoOverlay) { 
        formData.append('speedoPosition', overlayConfig.speedoPosition); 
    }
    
    formData.append('addTrackOverlay', overlayConfig.addTrackOverlay);
    if (overlayConfig.addTrackOverlay) { 
        formData.append('trackPosition', overlayConfig.trackPosition); 
    }
    
    formData.append('addStatsOverlay', overlayConfig.addStatsOverlay);
    if (overlayConfig.addStatsOverlay) { 
        formData.append('statsPosition', overlayConfig.statsPosition); 
    }

    const xhr = new XMLHttpRequest();
    xhr.open('POST', '/process', true);
    
    // Monitorar progresso de upload
    xhr.upload.onprogress = (event) => { 
        if (event.lengthComputable) { 
            const uploadPercentage = Math.round((event.loaded / event.total) * 15); // Upload = 15% do total
            loader.setProgress(uploadPercentage);
        } 
    };
    
    xhr.onload = () => {
        loader.hide();
        
        const result = JSON.parse(xhr.responseText);
        logsPre.textContent = result.logs.join('\n');
        logsContainer.style.display = 'block';
        
        if (xhr.status >= 200 && xhr.status < 300) {
            notify.success(t('notification_success'), t('success_message'));
            if (result.download_url) { 
                downloadBtn.href = result.download_url;
                downloadBtn.classList.add('show'); // Mostrar botão discreto
            }
        } else {
            notify.error(t('notification_error'), t('server_error', { message: result.message || 'Unknown error' }));
        }
        
        generateBtn.disabled = false;
        validateGenerateButton();
    };
    
    xhr.onerror = () => { 
        loader.hide();
        notify.error(t('notification_error'), t('network_error'));
        generateBtn.disabled = false; 
        validateGenerateButton();
    };
    
    xhr.send(formData);
}