// js/map-manager.js - Gerenciamento do mapa com coordenadas corrigidas

// Variáveis de estado do mapa
let map = null;
let trackLayer = null;
let userMarker = null;
let suggestionMarker = null;
let gpxDataPoints = [];
let selectedSyncPoint = null;

// Inicialização do mapa
map = L.map('map').setView([0, 0], 2);
L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', { 
    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors' 
}).addTo(map);

// Ícones personalizados
const suggestionIcon = L.icon({ 
    iconUrl: 'data:image/svg+xml;base64,' + btoa(`<svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41"><path fill="#2E8B57" stroke="#FFFFFF" stroke-width="1.5" d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/><circle fill="#FFFFFF" cx="12.5" cy="12.5" r="4"/></svg>`), 
    iconSize: [25, 41], 
    iconAnchor: [12, 41], 
    popupAnchor: [1, -34] 
});

const userIcon = L.icon({ 
    iconUrl: 'data:image/svg+xml;base64=' + btoa(`<svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41"><path fill="#03dac6" stroke="#121212" stroke-width="1" d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/><circle fill="#121212" cx="12.5" cy="12.5" r="4"/></svg>`), 
    iconSize: [25, 41], 
    iconAnchor: [12, 41], 
    popupAnchor: [1, -34] 
});

function displayTrack(points) {
    console.log('🗺️ Exibindo trilha com', points.length, 'pontos');
    
    if (trackLayer) {
        map.removeLayer(trackLayer); 
    }
    
    if (!points || points.length === 0) {
        console.warn('⚠️ Nenhum ponto para exibir');
        return;
    }
    
    // Validar e filtrar pontos válidos
    const validPoints = points.filter(p => {
        const lat = parseFloat(p.lat);
        const lon = parseFloat(p.lon);
        
        // Verificar se as coordenadas estão dentro de limites válidos
        const isValidLat = !isNaN(lat) && lat >= -90 && lat <= 90;
        const isValidLon = !isNaN(lon) && lon >= -180 && lon <= 180;
        
        if (!isValidLat || !isValidLon) {
            console.warn('⚠️ Coordenada inválida:', { lat, lon, original: p });
            return false;
        }
        
        return true;
    });
    
    console.log(`✅ ${validPoints.length}/${points.length} pontos válidos`);
    
    if (validPoints.length === 0) {
        notify.error('Erro no Mapa', 'Nenhuma coordenada válida encontrada na trilha');
        return;
    }
    
    // Converter para formato Leaflet [lat, lng] - ORDEM CORRETA
    const latLngs = validPoints.map(p => {
        const lat = parseFloat(p.lat);
        const lon = parseFloat(p.lon);
        return [lat, lon]; // Leaflet usa [latitude, longitude]
    });
    
    console.log('🗺️ Primeira coordenada:', latLngs[0]);
    console.log('🗺️ Última coordenada:', latLngs[latLngs.length - 1]);
    
    // Calcular bounds da trilha
    const bounds = L.latLngBounds(latLngs);
    console.log('🗺️ Bounds calculados:', bounds);
    
    // Verificar se bounds são válidos (não cobrem continentes inteiros)
    const boundsSize = {
        lat: bounds.getNorth() - bounds.getSouth(),
        lng: bounds.getEast() - bounds.getWest()
    };
    
    if (boundsSize.lat > 180 || boundsSize.lng > 360) {
        console.error('❌ Bounds suspeitos - possível problema de coordenadas:', boundsSize);
        notify.error('Erro no Mapa', 'Coordenadas da trilha parecem estar incorretas');
        return;
    }
    
    // Criar trilha colorida por velocidade
    const trackSegments = [];
    
    for (let i = 0; i < validPoints.length - 1; i++) {
        const p1 = validPoints[i];
        const p2 = validPoints[i + 1];
        
        const lat1 = parseFloat(p1.lat);
        const lon1 = parseFloat(p1.lon);
        const lat2 = parseFloat(p2.lat);
        const lon2 = parseFloat(p2.lon);
        
        // Calcular velocidade se disponível
        let speed = 0;
        if (p1.speed !== undefined && p1.speed !== null) {
            speed = parseFloat(p1.speed) || 0;
        } else if (p1.time && p2.time) {
            // Calcular velocidade baseada na distância e tempo
            const distance = calculateDistance(lat1, lon1, lat2, lon2);
            const timeDiff = (new Date(p2.time) - new Date(p1.time)) / 1000; // segundos
            if (timeDiff > 0) {
                speed = (distance / timeDiff) * 3.6; // km/h
            }
        }
        
        // Cor baseada na velocidade
        const color = getSpeedColor(speed);
        
        // Criar segmento
        const segment = L.polyline([[lat1, lon1], [lat2, lon2]], {
            color: color,
            weight: 4,
            opacity: 0.8
        });
        
        trackSegments.push(segment);
    }
    
    // Criar grupo de camadas
    trackLayer = L.layerGroup(trackSegments).addTo(map);
    
    // Ajustar zoom para a trilha com padding
    if (bounds.isValid()) {
        map.fitBounds(bounds.pad(0.1));
        console.log('🗺️ Mapa ajustado para bounds:', bounds);
    } else {
        console.error('❌ Bounds inválidos');
    }
    
    // Adicionar evento de clique na trilha
    trackSegments.forEach((segment, index) => {
        segment.on('click', (e) => {
            console.log('🖱️ Clique na trilha:', e.latlng);
            
            // Encontrar ponto mais próximo
            let closestPoint = null;
            let minDistance = Infinity;
            
            validPoints.forEach(p => {
                const distance = map.distance([p.lat, p.lon], e.latlng);
                if (distance < minDistance) {
                    minDistance = distance;
                    closestPoint = p;
                }
            });
            
            if (closestPoint) {
                console.log('📍 Ponto mais próximo:', closestPoint);
                selectSyncPoint(closestPoint, false);
            }
        });
    });
    
    // Atualizar dados globais
    gpxDataPoints = validPoints;
    
    notify.success('Mapa', `Trilha exibida: ${validPoints.length} pontos`);
}

function selectSyncPoint(point, isSuggestion) {
    console.log('📍 Selecionando ponto:', point, 'Sugestão:', isSuggestion);
    
    selectedSyncPoint = point; 
    
    // Remover marcadores anteriores
    if (userMarker) {
        map.removeLayer(userMarker); 
        userMarker = null;
    }
    if (suggestionMarker) {
        map.removeLayer(suggestionMarker); 
        suggestionMarker = null;
    }
    
    // Validar coordenadas do ponto
    const lat = parseFloat(point.lat);
    const lon = parseFloat(point.lon);
    
    if (isNaN(lat) || isNaN(lon) || lat < -90 || lat > 90 || lon < -180 || lon > 180) {
        console.error('❌ Coordenadas inválidas para marcador:', { lat, lon });
        notify.error('Erro', 'Coordenadas do ponto de sincronização inválidas');
        return;
    }
    
    // Criar marcador
    const iconToUse = isSuggestion ? suggestionIcon : userIcon;
    const newMarker = L.marker([lat, lon], { icon: iconToUse }).addTo(map);
    
    // Adicionar popup informativo
    let popupContent = `<strong>Ponto de Sincronização</strong><br>`;
    popupContent += `Lat: ${lat.toFixed(6)}<br>`;
    popupContent += `Lng: ${lon.toFixed(6)}<br>`;
    
    if (point.time) {
        const time = new Date(point.time);
        popupContent += `Tempo: ${time.toLocaleString()}<br>`;
    }
    
    if (point.speed) {
        popupContent += `Velocidade: ${parseFloat(point.speed).toFixed(1)} km/h<br>`;
    }
    
    newMarker.bindPopup(popupContent);
    
    // Armazenar referência
    if (isSuggestion) { 
        suggestionMarker = newMarker; 
    } else { 
        userMarker = newMarker; 
    }
    
    // Atualizar UI
    const pointTime = point.displayTime || (point.time ? new Date(point.time).toLocaleString(currentLang.startsWith('en') ? 'en-US' : 'pt-BR', { timeZone: 'UTC' }) : 'Tempo não disponível');
    syncPointInfo.textContent = t('sync_point_selected', { 
        type: isSuggestion ? t('suggestion_type') : t('manual_type'), 
        time: pointTime 
    });
    
    if (!isSuggestion) {
        notify.success(t('notification_sync_selected'), `Coordenadas: ${lat.toFixed(6)}, ${lon.toFixed(6)}`);
    }
    
    // Mostrar seção de posicionamento
    if (videoFile) { 
        positionSection.style.display = 'block'; 
    }
    
    validateGenerateButton();
}

// Função auxiliar para calcular distância entre dois pontos (fórmula de Haversine)
function calculateDistance(lat1, lon1, lat2, lon2) {
    const R = 6371000; // Raio da Terra em metros
    const φ1 = lat1 * Math.PI/180;
    const φ2 = lat2 * Math.PI/180;
    const Δφ = (lat2-lat1) * Math.PI/180;
    const Δλ = (lon2-lon1) * Math.PI/180;

    const a = Math.sin(Δφ/2) * Math.sin(Δφ/2) +
              Math.cos(φ1) * Math.cos(φ2) *
              Math.sin(Δλ/2) * Math.sin(Δλ/2);
    const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1-a));

    return R * c; // em metros
}

// Função para determinar cor baseada na velocidade
function getSpeedColor(speed) {
    const maxSpeed = 50; // km/h para escala de cores
    const ratio = Math.min(speed / maxSpeed, 1);
    
    if (ratio < 0.2) {
        return '#0066cc'; // Azul (muito lento)
    } else if (ratio < 0.4) {
        return '#00cc66'; // Verde (lento)
    } else if (ratio < 0.6) {
        return '#cccc00'; // Amarelo (moderado)
    } else if (ratio < 0.8) {
        return '#ff6600'; // Laranja (rápido)
    } else {
        return '#ff0000'; // Vermelho (muito rápido)
    }
}

// Função para limpar o mapa
function clearMap() {
    if (trackLayer) {
        map.removeLayer(trackLayer);
        trackLayer = null;
    }
    
    if (userMarker) {
        map.removeLayer(userMarker);
        userMarker = null;
    }
    
    if (suggestionMarker) {
        map.removeLayer(suggestionMarker);
        suggestionMarker = null;
    }
    
    selectedSyncPoint = null;
    gpxDataPoints = [];
    
    // Reset zoom
    map.setView([0, 0], 2);
}

// Função para debug das coordenadas
function debugCoordinates(points) {
    console.group('🔍 Debug Coordenadas');
    console.log('Total de pontos:', points.length);
    
    if (points.length > 0) {
        const firstPoint = points[0];
        const lastPoint = points[points.length - 1];
        
        console.log('Primeiro ponto:', firstPoint);
        console.log('Último ponto:', lastPoint);
        
        // Verificar se as coordenadas fazem sentido geograficamente
        const lats = points.map(p => parseFloat(p.lat)).filter(lat => !isNaN(lat));
        const lons = points.map(p => parseFloat(p.lon)).filter(lon => !isNaN(lon));
        
        const latRange = { min: Math.min(...lats), max: Math.max(...lats) };
        const lonRange = { min: Math.min(...lons), max: Math.max(...lons) };
        
        console.log('Range de latitudes:', latRange);
        console.log('Range de longitudes:', lonRange);
        
        // Alertas para coordenadas suspeitas
        if (latRange.min === latRange.max && lonRange.min === lonRange.max) {
            console.warn('⚠️ Todos os pontos têm a mesma coordenada!');
        }
        
        if (Math.abs(latRange.max - latRange.min) > 90) {
            console.warn('⚠️ Range de latitude muito grande - possível erro!');
        }
        
        if (Math.abs(lonRange.max - lonRange.min) > 180) {
            console.warn('⚠️ Range de longitude muito grande - possível erro!');
        }
    }
    
    console.groupEnd();
}