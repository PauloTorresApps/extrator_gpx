export default class LoadingManager {
    constructor() {
        this.overlay = document.getElementById('loading-overlay');
        this.progressBar = document.getElementById('loading-progress-bar');
        this.percentage = document.getElementById('loading-percentage');
        this.cancelBtn = document.getElementById('loading-cancel');
        this.steps = ['step-upload', 'step-analysis', 'step-sync', 'step-overlays', 'step-render'];
        this.isVisible = false;
        this.onCancel = null;
        
        this.cancelBtn.addEventListener('click', () => {
            if (this.onCancel) this.onCancel();
            this.hide();
        });
    }
    
    show(onCancelCallback = null) {
        this.onCancel = onCancelCallback;
        this.isVisible = true;
        this.resetSteps();
        this.setProgress(0);
        this.overlay.classList.add('show');
    }
    
    hide() {
        this.isVisible = false;
        this.overlay.classList.remove('show');
    }
    
    setProgress(percentage) {
        const clampedPercentage = Math.max(0, Math.min(100, percentage));
        this.progressBar.style.width = `${clampedPercentage}%`;
        this.percentage.textContent = `${Math.round(clampedPercentage)}%`;
    }
    
    resetSteps() {
        this.steps.forEach((stepId, index) => {
            const stepElement = document.getElementById(stepId);
            stepElement.className = 'loading-step pending';
            stepElement.querySelector('.step-icon').textContent = index + 1;
        });
    }
    
    // A simulação de progresso foi removida daqui e será controlada pela API
    // para refletir o progresso real de forma mais precisa.
}