// js/map-manager.js - Gerenciamento do mapa com coordenadas CORRIGIDAS

// Variáveis de estado do mapa
let map = null;
let trackLayer = null;
let userMarker = null;
let suggestionMarker = null;
let gpxDataPoints = [];
let selectedSyncPoint = null;

// Inicialização do mapa - CORREÇÃO: Aguardar DOM estar pronto
function initializeMap() {
    if (map) return; // Evitar inicialização dupla
    
    const mapElement = document.getElementById('map');
    if (!mapElement) {
        console.warn('⚠️ Elemento do mapa não encontrado, aguardando...');
        setTimeout(initializeMap, 500);
        return;
    }
    
    try {
        map = L.map('map').setView([0, 0], 2);
        L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', { 
            attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors' 
        }).addTo(map);
        
        console.log('🗺️ Mapa inicializado com sucesso');
    } catch (error) {
        console.error('❌ Erro ao inicializar mapa:', error);
    }
}

// Chamar inicialização quando DOM estiver pronto
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeMap);
} else {
    initializeMap();
}

// Ícones personalizados - CORREÇÃO: Melhorar definição dos ícones
const suggestionIcon = L.icon({ 
    iconUrl: 'data:image/svg+xml;base64,' + btoa(`
        <svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41">
            <path fill="#2E8B57" stroke="#FFFFFF" stroke-width="2" 
                  d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/>
            <circle fill="#FFFFFF" cx="12.5" cy="12.5" r="6"/>
            <text x="12.5" y="17" text-anchor="middle" fill="#2E8B57" font-size="8" font-weight="bold">S</text>
        </svg>
    `), 
    iconSize: [25, 41], 
    iconAnchor: [12, 41], 
    popupAnchor: [1, -34] 
});

const userIcon = L.icon({ 
    iconUrl: 'data:image/svg+xml;base64=' + btoa(`
        <svg xmlns="http://www.w3.org/2000/svg" width="25" height="41" viewBox="0 0 25 41">
            <path fill="#03dac6" stroke="#121212" stroke-width="2" 
                  d="M12.5 0C5.6 0 0 5.6 0 12.5c0 12.5 12.5 28.5 12.5 28.5s12.5-16 12.5-28.5C25 5.6 19.4 0 12.5 0z"/>
            <circle fill="#121212" cx="12.5" cy="12.5" r="6"/>
            <text x="12.5" y="17" text-anchor="middle" fill="#03dac6" font-size="8" font-weight="bold">M</text>
        </svg>
    `), 
    iconSize: [25, 41], 
    iconAnchor: [12, 41], 
    popupAnchor: [1, -34] 
});

function displayTrack(points) {
    console.log('🗺️ INÍCIO - Exibindo trilha com', points.length, 'pontos');
    
    // CORREÇÃO: Verificar se o mapa foi inicializado
    if (!map) {
        console.error('❌ Mapa não inicializado, tentando inicializar...');
        initializeMap();
        setTimeout(() => displayTrack(points), 1000);
        return;
    }
    
    // Remover trilha anterior
    if (trackLayer) {
        map.removeLayer(trackLayer); 
        trackLayer = null;
    }
    
    if (!points || points.length === 0) {
        console.warn('⚠️ Nenhum ponto para exibir');
        return;
    }
    
    // CORREÇÃO: Validação mais rigorosa das coordenadas
    const validPoints = [];
    let invalidCount = 0;
    
    points.forEach((p, index) => {
        // Tentar diferentes formatos de coordenadas
        let lat, lon;
        
        if (typeof p.lat !== 'undefined' && typeof p.lon !== 'undefined') {
            lat = parseFloat(p.lat);
            lon = parseFloat(p.lon);
        } else if (typeof p.latitude !== 'undefined' && typeof p.longitude !== 'undefined') {
            lat = parseFloat(p.latitude);
            lon = parseFloat(p.longitude);
        } else if (Array.isArray(p) && p.length >= 2) {
            lat = parseFloat(p[0]);
            lon = parseFloat(p[1]);
        } else {
            console.warn(`⚠️ Ponto ${index}: formato desconhecido:`, p);
            invalidCount++;
            return;
        }
        
        // Validação rigorosa das coordenadas
        if (isNaN(lat) || isNaN(lon)) {
            console.warn(`⚠️ Ponto ${index}: coordenadas NaN - lat: ${lat}, lon: ${lon}`);
            invalidCount++;
            return;
        }
        
        if (lat < -90 || lat > 90) {
            console.warn(`⚠️ Ponto ${index}: latitude inválida: ${lat}`);
            invalidCount++;
            return;
        }
        
        if (lon < -180 || lon > 180) {
            console.warn(`⚠️ Ponto ${index}: longitude inválida: ${lon}`);
            invalidCount++;
            return;
        }
        
        // Filtrar pontos (0,0) que são suspeitos
        if (lat === 0 && lon === 0) {
            console.warn(`⚠️ Ponto ${index}: coordenada (0,0) ignorada`);
            invalidCount++;
            return;
        }
        
        validPoints.push({
            ...p,
            lat: lat,
            lon: lon,
            original_index: index
        });
    });
    
    console.log(`✅ Pontos válidos: ${validPoints.length}/${points.length} (${invalidCount} inválidos)`);
    
    if (validPoints.length === 0) {
        notify.error('Erro no Mapa', 'Nenhuma coordenada válida encontrada na trilha');
        return;
    }
    
    if (validPoints.length < 2) {
        notify.warning('Aviso', 'Trilha tem apenas um ponto válido. Não é possível traçar linha.');
        // Mesmo assim, mostrar o ponto único
    }
    
    // CORREÇÃO: Construção das coordenadas Leaflet
    const latLngs = validPoints.map(p => [p.lat, p.lon]); // [latitude, longitude]
    
    console.log('🗺️ Primeira coordenada:', latLngs[0]);
    console.log('🗺️ Última coordenada:', latLngs[latLngs.length - 1]);
    
    // Calcular bounds da trilha
    const lats = validPoints.map(p => p.lat);
    const lons = validPoints.map(p => p.lon);
    
    const bounds = {
        north: Math.max(...lats),
        south: Math.min(...lats),
        east: Math.max(...lons),
        west: Math.min(...lons)
    };
    
    console.log('🗺️ Bounds calculados:', bounds);
    
    // CORREÇÃO: Verificar se bounds são válidos
    const boundsSize = {
        lat: bounds.north - bounds.south,
        lng: bounds.east - bounds.west
    };
    
    if (boundsSize.lat > 180 || boundsSize.lng > 360) {
        console.error('❌ Bounds suspeitos - possível problema de coordenadas:', boundsSize);
        notify.error('Erro no Mapa', 'Coordenadas da trilha parecem estar incorretas');
        return;
    }
    
    if (boundsSize.lat < 0.000001 && boundsSize.lng < 0.000001) {
        console.warn('⚠️ Todos os pontos estão muito próximos, usando zoom alto');
    }
    
    // Criar trilha colorida por velocidade - CORREÇÃO
    const trackSegments = [];
    
    // Se há apenas um ponto, criar um marcador
    if (validPoints.length === 1) {
        const point = validPoints[0];
        const marker = L.circleMarker([point.lat, point.lon], {
            radius: 8,
            fillColor: '#ff0000',
            color: '#ffffff',
            weight: 2,
            opacity: 1,
            fillOpacity: 0.8
        });
        trackSegments.push(marker);
    } else {
        // Criar segmentos da trilha
        for (let i = 0; i < validPoints.length - 1; i++) {
            const p1 = validPoints[i];
            const p2 = validPoints[i + 1];
            
            // Calcular velocidade se disponível
            let speed = 0;
            if (p1.speed !== undefined && p1.speed !== null) {
                speed = parseFloat(p1.speed) || 0;
            } else if (p1.time && p2.time) {
                // Calcular velocidade baseada na distância e tempo
                const distance = calculateDistance(p1.lat, p1.lon, p2.lat, p2.lon);
                const time1 = new Date(p1.time);
                const time2 = new Date(p2.time);
                const timeDiff = (time2 - time1) / 1000; // segundos
                if (timeDiff > 0) {
                    speed = (distance / timeDiff) * 3.6; // km/h
                }
            }
            
            // Cor baseada na velocidade
            const color = getSpeedColor(speed);
            
            // CORREÇÃO: Criar segmento com coordenadas corretas
            const segment = L.polyline([[p1.lat, p1.lon], [p2.lat, p2.lon]], {
                color: color,
                weight: 4,
                opacity: 0.8
            });
            
            // CORREÇÃO: Adicionar popup com informações do segmento
            segment.bindPopup(`
                <strong>Segmento ${i + 1}</strong><br>
                Velocidade: ${speed.toFixed(1)} km/h<br>
                Coords: ${p1.lat.toFixed(6)}, ${p1.lon.toFixed(6)}
            `);
            
            trackSegments.push(segment);
        }
    }
    
    // CORREÇÃO: Criar grupo de camadas
    trackLayer = L.layerGroup(trackSegments).addTo(map);
    
    // CORREÇÃO: Ajustar zoom para a trilha com padding
    const leafletBounds = L.latLngBounds(latLngs);
    if (leafletBounds.isValid()) {
        if (validPoints.length === 1) {
            // Para um único ponto, centralizar com zoom alto
            map.setView([validPoints[0].lat, validPoints[0].lon], 15);
        } else {
            // Para múltiplos pontos, ajustar bounds
            map.fitBounds(leafletBounds.pad(0.1));
        }
        console.log('🗺️ Mapa ajustado para bounds:', leafletBounds);
    } else {
        console.error('❌ Bounds inválidos');
        // Fallback: centrar no primeiro ponto
        if (validPoints.length > 0) {
            map.setView([validPoints[0].lat, validPoints[0].lon], 10);
        }
    }
    
    // CORREÇÃO: Adicionar evento de clique na trilha e pontos
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
    
    notify.success('Mapa', `Trilha exibida: ${validPoints.length} pontos válidos`);
    console.log('🗺️ FIM - Trilha exibida com sucesso');
}

function selectSyncPoint(point, isSuggestion = false) {
    console.log('📍 INÍCIO - Selecionando ponto:', point, 'Sugestão:', isSuggestion);
    
    // CORREÇÃO: Verificar se o mapa foi inicializado
    if (!map) {
        console.error('❌ Mapa não inicializado para marcar ponto');
        return;
    }
    
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
    
    // CORREÇÃO: Extrair coordenadas de forma mais robusta
    let lat, lon;
    
    if (typeof point.lat !== 'undefined' && typeof point.lon !== 'undefined') {
        lat = parseFloat(point.lat);
        lon = parseFloat(point.lon);
    } else if (typeof point.latitude !== 'undefined' && typeof point.longitude !== 'undefined') {
        lat = parseFloat(point.latitude);
        lon = parseFloat(point.longitude);
    } else {
        console.error('❌ Formato de coordenadas não reconhecido:', point);
        return;
    }
    
    // Validar coordenadas do ponto
    if (isNaN(lat) || isNaN(lon) || lat < -90 || lat > 90 || lon < -180 || lon > 180) {
        console.error('❌ Coordenadas inválidas para marcador:', { lat, lon });
        notify.error('Erro', 'Coordenadas do ponto de sincronização inválidas');
        return;
    }
    
    console.log('📍 Coordenadas validadas:', { lat, lon });
    
    // Criar marcador
    const iconToUse = isSuggestion ? suggestionIcon : userIcon;
    const newMarker = L.marker([lat, lon], { icon: iconToUse }).addTo(map);
    
    // CORREÇÃO: Adicionar popup mais informativo
    let popupContent = `
        <div style="min-width: 200px;">
            <strong>${isSuggestion ? '🎯 Ponto Sugerido' : '📍 Ponto Manual'}</strong><br>
            <strong>Coordenadas:</strong><br>
            Lat: ${lat.toFixed(6)}<br>
            Lng: ${lon.toFixed(6)}<br>
    `;
    
    if (point.time) {
        const time = new Date(point.time);
        if (!isNaN(time.getTime())) {
            popupContent += `<strong>Tempo:</strong><br>${time.toLocaleString()}<br>`;
        }
    }
    
    if (point.speed && !isNaN(point.speed)) {
        popupContent += `<strong>Velocidade:</strong> ${parseFloat(point.speed).toFixed(1)} km/h<br>`;
    }
    
    if (point.heart_rate && !isNaN(point.heart_rate)) {
        popupContent += `<strong>FC:</strong> ${Math.round(point.heart_rate)} bpm<br>`;
    }
    
    popupContent += '</div>';
    
    newMarker.bindPopup(popupContent);
    
    // Armazenar referência
    if (isSuggestion) { 
        suggestionMarker = newMarker; 
    } else { 
        userMarker = newMarker; 
    }
    
    // CORREÇÃO: Atualizar UI
    const syncPointInfo = document.getElementById('sync-point-info');
    if (syncPointInfo) {
        const pointTime = point.displayTime || 
            (point.time ? new Date(point.time).toLocaleString(
                currentLang.startsWith('en') ? 'en-US' : 'pt-BR', 
                { timeZone: 'UTC' }
            ) : 'Tempo não disponível');
            
        syncPointInfo.textContent = t('sync_point_selected', { 
            type: isSuggestion ? t('suggestion_type') : t('manual_type'), 
            time: pointTime 
        });
    }
    
    if (!isSuggestion) {
        notify.success(t('notification_sync_selected'), 
            `Coordenadas: ${lat.toFixed(6)}, ${lon.toFixed(6)}`);
    }
    
    // CORREÇÃO: Centralizar no ponto selecionado
    map.setView([lat, lon], Math.max(map.getZoom(), 12));
    
    // Mostrar seção de posicionamento
    const positionSection = document.getElementById('position-section');
    const videoFile = window.videoFile; // Referência global
    if (positionSection && videoFile) { 
        positionSection.style.display = 'block'; 
    }
    
    // Validar botão de gerar
    if (typeof validateGenerateButton === 'function') {
        validateGenerateButton();
    }
    
    console.log('📍 FIM - Ponto selecionado com sucesso');
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
    if (!map) return;
    
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

// CORREÇÃO: Função para debug das coordenadas aprimorada
function debugCoordinates(points) {
    console.group('🔍 Debug Coordenadas DETALHADO');
    console.log('Total de pontos:', points.length);
    
    if (points.length > 0) {
        const sampleSize = Math.min(5, points.length);
        console.log(`Amostra dos primeiros ${sampleSize} pontos:`);
        
        for (let i = 0; i < sampleSize; i++) {
            const p = points[i];
            console.log(`Ponto ${i}:`, {
                estrutura: Object.keys(p),
                lat: p.lat || p.latitude,
                lon: p.lon || p.longitude,
                time: p.time,
                speed: p.speed,
                original: p
            });
        }
        
        // Análise estatística das coordenadas
        const lats = points
            .map(p => parseFloat(p.lat || p.latitude))
            .filter(lat => !isNaN(lat));
        const lons = points
            .map(p => parseFloat(p.lon || p.longitude))
            .filter(lon => !isNaN(lon));
        
        if (lats.length > 0 && lons.length > 0) {
            const latStats = {
                min: Math.min(...lats),
                max: Math.max(...lats),
                avg: lats.reduce((a, b) => a + b, 0) / lats.length
            };
            const lonStats = {
                min: Math.min(...lons),
                max: Math.max(...lons),
                avg: lons.reduce((a, b) => a + b, 0) / lons.length
            };
            
            console.log('Estatísticas de latitude:', latStats);
            console.log('Estatísticas de longitude:', lonStats);
            
            // Detectar problemas comuns
            if (latStats.min === latStats.max && lonStats.min === lonStats.max) {
                console.warn('🚨 PROBLEMA: Todos os pontos têm a mesma coordenada!');
            }
            
            if (Math.abs(latStats.max - latStats.min) > 10) {
                console.warn('🚨 PROBLEMA: Range de latitude muito grande (>10°)');
            }
            
            if (Math.abs(lonStats.max - lonStats.min) > 10) {
                console.warn('🚨 PROBLEMA: Range de longitude muito grande (>10°)');
            }
            
            // Identificar região geográfica
            const region = identifyRegion(latStats.avg, lonStats.avg);
            console.log('📍 Região detectada:', region);
        }
    }
    
    console.groupEnd();
}

function identifyRegion(lat, lon) {
    // Identificar região geográfica aproximada
    if (lat >= -35.0 && lat <= 5.0 && lon >= -75.0 && lon <= -30.0) {
        return 'Brasil';
    } else if (lat >= 24.0 && lat <= 49.0 && lon >= -125.0 && lon <= -66.0) {
        return 'Estados Unidos';
    } else if (lat >= 35.0 && lat <= 71.0 && lon >= -10.0 && lon <= 40.0) {
        return 'Europa';
    } else if (lat >= -90.0 && lat <= -60.0) {
        return 'Antártica';
    } else if (lat >= 60.0 && lat <= 90.0) {
        return 'Ártico';
    } else if (Math.abs(lat) < 0.001 && Math.abs(lon) < 0.001) {
        return 'Coordenadas (0,0) - ERRO SUSPEITO';
    } else {
        return `Região não identificada (lat: ${lat.toFixed(2)}, lon: ${lon.toFixed(2)})`;
    }
}

// NOVO: Função para testar o mapa com dados sintéticos
function testMapWithSyntheticData() {
    console.log('🧪 Testando mapa com dados sintéticos...');
    
    const testPoints = [
        { lat: -15.7939, lon: -47.8828, time: new Date(), speed: 10 }, // Brasília
        { lat: -15.7949, lon: -47.8838, time: new Date(), speed: 15 },
        { lat: -15.7959, lon: -47.8848, time: new Date(), speed: 20 },
        { lat: -15.7969, lon: -47.8858, time: new Date(), speed: 12 },
    ];
    
    console.log('📍 Pontos de teste:', testPoints);
    displayTrack(testPoints);
    
    setTimeout(() => {
        selectSyncPoint(testPoints[1], true);
    }, 1000);
}

// Expor função de teste globalmente para debug
window.testMapWithSyntheticData = testMapWithSyntheticData;
window.debugCoordinates = debugCoordinates;