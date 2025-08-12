// js/loading.js - Sistema de loading avançado

class LoadingManager {
    constructor() {
        this.overlay = document.getElementById('loading-overlay');
        this.progressBar = document.getElementById('loading-progress-bar');
        this.percentage = document.getElementById('loading-percentage');
        this.cancelBtn = document.getElementById('loading-cancel');
        this.steps = [
            'step-upload',
            'step-analysis', 
            'step-sync',
            'step-overlays',
            'step-render'
        ];
        this.currentStep = 0;
        this.isVisible = false;
        this.onCancel = null;
        
        this.cancelBtn.addEventListener('click', () => {
            if (this.onCancel) {
                this.onCancel();
            }
            this.hide();
        });
    }
    
    show(onCancelCallback = null) {
        this.onCancel = onCancelCallback;
        this.isVisible = true;
        this.currentStep = 0;
        this.resetSteps();
        this.setProgress(0);
        this.overlay.classList.add('show');
    }
    
    hide() {
        this.isVisible = false;
        this.overlay.classList.remove('show');
        this.resetSteps();
        this.setProgress(0);
    }
    
    setStep(stepIndex, status = 'active') {
        if (stepIndex >= 0 && stepIndex < this.steps.length) {
            const stepElement = document.getElementById(this.steps[stepIndex]);
            const stepIcon = stepElement.querySelector('.step-icon');
            
            // Limpar classes anteriores
            stepElement.className = `loading-step ${status}`;
            
            // Atualizar ícone baseado no status
            if (status === 'completed') {
                stepIcon.textContent = '✓';
            } else if (status === 'error') {
                stepIcon.textContent = '✗';
            } else {
                stepIcon.textContent = stepIndex + 1;
            }
            
            this.currentStep = stepIndex;
        }
    }
    
    nextStep(status = 'completed') {
        if (this.currentStep < this.steps.length) {
            this.setStep(this.currentStep, status);
            if (this.currentStep + 1 < this.steps.length) {
                this.setStep(this.currentStep + 1, 'active');
            }
        }
    }
    
    setProgress(percentage) {
        const clampedPercentage = Math.max(0, Math.min(100, percentage));
        this.progressBar.style.width = `${clampedPercentage}%`;
        this.percentage.textContent = `${Math.round(clampedPercentage)}%`;
    }
    
    resetSteps() {
        this.steps.forEach((stepId, index) => {
            const stepElement = document.getElementById(stepId);
            const stepIcon = stepElement.querySelector('.step-icon');
            stepElement.className = 'loading-step pending';
            stepIcon.textContent = index + 1;
        });
    }
    
    simulateProgress() {
        // Simulação realística do progresso baseada nas etapas
        const progressSteps = [
            { step: 0, progress: 15 },  // Upload
            { step: 1, progress: 35 },  // Análise
            { step: 2, progress: 55 },  // Sync
            { step: 3, progress: 80 },  // Overlays
            { step: 4, progress: 100 }  // Render
        ];
        
        let currentProgressIndex = 0;
        
        const updateProgress = () => {
            if (!this.isVisible || currentProgressIndex >= progressSteps.length) {
                return;
            }
            
            const currentTarget = progressSteps[currentProgressIndex];
            this.setStep(currentTarget.step, 'active');
            this.setProgress(currentTarget.progress);
            
            // Marcar etapa anterior como completa
            if (currentProgressIndex > 0) {
                this.setStep(progressSteps[currentProgressIndex - 1].step, 'completed');
            }
            
            currentProgressIndex++;
            
            // Timing realístico para cada etapa
            const delays = [3000, 4000, 3000, 5000, 6000]; // ms
            if (currentProgressIndex < progressSteps.length) {
                setTimeout(updateProgress, delays[currentProgressIndex - 1]);
            } else {
                // Completar última etapa
                setTimeout(() => {
                    if (this.isVisible) {
                        this.setStep(4, 'completed');
                    }
                }, 1000);
            }
        };
        
        updateProgress();
    }
}

// Instância global do gerenciador de loading
const loader = new LoadingManager();