# GPX/TCX Video Sync & Overlay

Sistema para sincronização de vídeos com dados de trilhas GPS/TCX e aplicação de overlays de telemetria.

## 🆕 Nova Funcionalidade: Suporte TCX

Esta versão agora suporta arquivos **TCX (Training Center XML)** além dos arquivos GPX tradicionais, oferecendo funcionalidades aprimoradas de telemetria.

### O que é TCX?

TCX (Training Center XML) é um formato de dados criado pela Garmin que vai além do GPX simples:

- **Mais Dados de Telemetria**: Frequência cardíaca, cadência, calorias queimadas, velocidade
- **Informações de Atividade**: Tipo de esporte, voltas (laps), intensidade
- **Melhor Precisão**: Dados específicos para atividades esportivas
- **Compatibilidade**: Funciona com dispositivos Garmin, Strava, e outras plataformas

### Diferenças TCX vs GPX

| Característica | GPX | TCX |
|---|---|---|
| Coordenadas GPS | ✅ | ✅ |
| Elevação | ✅ | ✅ |
| Timestamp | ✅ | ✅ |
| Frequência Cardíaca | ❌ | ✅ |
| Cadência | ❌ | ✅ |
| Calorias | ❌ | ✅ |
| Tipo de Esporte | ❌ | ✅ |
| Dados de Voltas | ❌ | ✅ |

## 🚀 Funcionalidades

### Formatos Suportados
- **GPX**: Formato padrão para trilhas GPS
- **TCX**: Formato Garmin com telemetria avançada
- **Vídeos**: MP4, AVI, MOV, e outros formatos populares

### Overlays Disponíveis
1. **Velocímetro**: Mostra velocidade atual, direção, força G e elevação
2. **Mapa do Trajeto**: Mini-mapa com trilha percorrida e posição atual
3. **Estatísticas**: Painel com distância, altitude, ganho de elevação, tempo e data

### Recursos TCX Específicos
- **Detecção Automática**: Identifica automaticamente arquivos TCX
- **Dados Extras**: Exibe frequência cardíaca, cadência e calorias quando disponíveis
- **Tipo de Esporte**: Detecta e exibe o tipo de atividade (corrida, ciclismo, etc.)
- **Estatísticas Avançadas**: Valores médios e máximos para telemetria

## 📋 Requisitos

### Sistema
- **Rust** 1.70+ com Cargo
- **FFmpeg** (para processamento de vídeo)
- **Node.js** (opcional, para desenvolvimento frontend)

### Dependências Rust Adicionadas
```toml
tcx = "0.9.3"  # Parser TCX
```

## 🛠 Instalação

1. **Clone o repositório**:
```bash
git clone <seu-repositorio>
cd extrator_gpx
```

2. **Instale o FFmpeg**:
   
   **Ubuntu/Debian**:
   ```bash
   sudo apt update
   sudo apt install ffmpeg
   ```
   
   **macOS**:
   ```bash
   brew install ffmpeg
   ```
   
   **Windows**:
   - Baixe do [site oficial do FFmpeg](https://ffmpeg.org/download.html)
   - Adicione ao PATH do sistema

3. **Compile e execute**:
```bash
cargo run
```

4. **Acesse a aplicação**:
   - Abra o navegador em `http://localhost:3030`

## 📖 Como Usar

### 1. Preparar Arquivos
- **Trilha**: Exporte sua atividade em formato TCX ou GPX
  - **Garmin Connect**: Activities → Export → TCX
  - **Strava**: Activities → Actions → Export as TCX
  - **Outros**: Certifique-se que o arquivo tem coordenadas GPS e timestamps

- **Vídeo**: Grave com timestamp correto (câmeras modernas fazem isso automaticamente)

### 2. Upload dos Arquivos
1. Selecione o arquivo de trilha (GPX/TCX)
2. Selecione o arquivo de vídeo
3. Se for TCX, você verá informações extras como:
   - Tipo de esporte detectado
   - Frequência cardíaca média/máxima
   - Cadência média/máxima
   - Calorias totais

### 3. Sincronização
- O sistema sugere automaticamente um ponto de sincronização
- Ajuste manualmente clicando no mapa se necessário
- A precisão é melhorada com interpolação de pontos

### 4. Configuração dos Overlays
- Ative/desative overlays clicando nas imagens
- Escolha a posição (cantos da tela)
- Configure interpolação nas configurações avançadas

### 5. Processamento
- Clique em "Confirmar e Gerar Vídeo"
- Acompanhe o progresso em tempo real
- Baixe o vídeo final quando pronto

## 🔧 Configurações Avançadas

### Interpolação de Pontos
- **1 segundo**: Máxima precisão (mais lento)
- **2-3 segundos**: Balanceado
- **4-5 segundos**: Mais rápido (menos preciso)

### Posicionamento de Overlays
- **Superior Esquerdo/Direito**
- **Inferior Esquerdo/Direito**
- Controle visual intuitivo

## 🏗 Arquitetura do Código

### Novos Módulos
```
src/
├── tcx_adapter.rs      # Conversão TCX → GPX + dados extras
├── main.rs             # Backend com suporte TCX
├── processing.rs       # Processamento evolutivo
└── ...
```

### Frontend Atualizado
```
static/js/
├── translations.js     # Traduções com termos TCX
├── file-handlers.js    # Upload e detecção TCX/GPX
├── map-manager.js      # Exibição de dados extras
└── ...
```

## 🔍 Detalhes Técnicos

### Conversão TCX → GPX
O sistema converte automaticamente arquivos TCX para o formato GPX interno, preservando:
- **Coordenadas GPS** (latitude/longitude)
- **Elevação** e **timestamps**
- **Dados extras** armazenados como comentários
- **Metadados** da atividade

### Dados TCX Suportados
- **Heart Rate**: Frequência cardíaca em BPM
- **Cadence**: Cadência de corrida/ciclismo
- **Speed**: Velocidade instantânea
- **Calories**: Calorias queimadas
- **Sport Type**: Tipo de atividade (Running, Cycling, etc.)
- **Lap Data**: Informações de voltas

### Formato de Armazenamento Interno
Os dados TCX extras são preservados como comentários no GPX:
```
HR:150;Cadence:90;Speed:15.5
```

## 🌍 Idiomas Suportados

- **Português (PT-BR)**: Idioma padrão
- **Inglês (EN)**: Tradução completa
- Termos específicos TCX traduzidos

## 🐛 Resolução de Problemas

### Erros Comuns

**"Formato de arquivo não suportado"**
- Verifique se o arquivo tem extensão `.tcx` ou `.gpx`
- Certifique-se que o arquivo não está corrompido

**"Nenhum ponto da trilha coincidiu com o tempo do vídeo"**
- Verifique se o timestamp do vídeo está correto
- Ajuste manualmente o ponto de sincronização
- Use um arquivo TCX/GPX com timestamps válidos

**"Erro ao ler arquivo TCX"**
- Arquivo pode estar mal formatado
- Tente exportar novamente da plataforma original
- Verifique se contém dados de GPS válidos

### Logs de Debug
Os logs mostram o progresso detalhado:
```
Lendo arquivo de trilha: "activity.tcx"
Arquivo de trilha lido com sucesso!
Tipo detectado: TCX
Esporte: Running
Pontos originais: 1,245
Pontos após interpolação: 3,735
```

## 📊 Exemplos de Uso

### Corrida com TCX
```
Arquivo: morning_run.tcx
Tipo: Running
Duração: 32:15
Distância: 5.2 km
Freq. Cardíaca: 145/178 bpm
Cadência: 165/185 spm
Calorias: 287 kcal
```

### Ciclismo com GPX
```
Arquivo: bike_ride.gpx
Tipo: Cycling (detectado automaticamente)
Duração: 1:45:30
Distância: 42.8 km
Elevação: +650m
```

## 🔄 Migração de Projetos Existentes

### Compatibilidade Retroativa
- **100% compatível** com projetos GPX existentes
- **Nenhuma alteração** necessária em fluxos GPX
- **Funcionalidades adicionais** apenas para TCX

### Atualização Gradual
1. Mantenha seus workflows GPX atuais
2. Experimente com arquivos TCX
3. Aproveite os dados extras quando disponíveis

## 🚀 Próximas Funcionalidades

### Em Desenvolvimento
- [ ] **Overlay de Frequência Cardíaca**: Gráfico em tempo real
- [ ] **Zonas de Treino**: Visualização de zonas cardíacas
- [ ] **Análise de Potência**: Para ciclismo (se disponível)
- [ ] **Comparação de Voltas**: Estatísticas por segmento

### Formatos Futuros
- [ ] **FIT Files**: Formato nativo Garmin
- [ ] **KML/KMZ**: Google Earth
- [ ] **PWX**: PeaksWare

## 🤝 Contribuindo

### Como Contribuir
1. Fork o repositório
2. Crie uma branch para sua feature
3. Implemente com testes
4. Abra um Pull Request

### Áreas de Interesse
- **Novos Formatos**: Implementação de parsers
- **Overlays**: Novos tipos de visualização
- **Performance**: Otimizações de processamento
- **UX**: Melhorias na interface

## 📄 Licença

Este projeto está sob a licença [MIT](LICENSE).

## 🆘 Suporte

### Documentação
- **Wiki**: Exemplos detalhados
- **Issues**: Reporte bugs e sugestões
- **Discussions**: Dúvidas e ideias

### Recursos Úteis
- [Especificação TCX](https://www8.garmin.com/xmlschemas/TrainingCenterDatabasev2.xsd)
- [Formato GPX](https://www.topografix.com/gpx.asp)
- [FFmpeg Docs](https://ffmpeg.org/documentation.html)

---

## 🎯 Resumo das Melhorias TCX

Esta atualização transforma o **GPX Video Sync** em uma ferramenta completa de telemetria esportiva:

### ✅ O que foi Adicionado
- **Suporte completo a TCX** com conversão automática
- **Dados de telemetria avançados** (HR, cadência, calorias)
- **Interface atualizada** com informações extras
- **Detecção automática** de tipo de arquivo
- **Traduções expandidas** para novos termos

### ✅ O que foi Preservado
- **100% compatibilidade** com GPX
- **Todos os overlays** existentes funcionam
- **Interface familiar** sem quebras
- **Performance** mantida ou melhorada

### ✅ Benefícios para Usuários
- **Mais dados** para análise esportiva
- **Melhor precisão** com dispositivos Garmin
- **Workflow unificado** para GPX e TCX
- **Preparação futura** para mais formatos

Esta é uma evolução natural que expande as capacidades do sistema mantendo a simplicidade de uso!