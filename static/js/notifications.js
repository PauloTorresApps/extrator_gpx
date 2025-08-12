// js/notifications.js - Sistema de notificações

class NotificationManager {
    constructor() {
        this.container = document.getElementById('notifications-container');
        this.notifications = new Map();
        this.notificationCounter = 0;
    }

    show(type, title, message, options = {}) {
        const id = `notification-${++this.notificationCounter}`;
        const duration = options.duration || 5000;
        const persistent = options.persistent || false;

        const icons = {
            success: '✅',
            error: '❌', 
            warning: '⚠️',
            info: 'ℹ️'
        };

        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        notification.id = id;
        notification.innerHTML = `
            <div class="notification-header">
                <div class="notification-icon">${icons[type] || '📋'}</div>
                <div class="notification-title">${title}</div>
                <button class="notification-close" aria-label="Fechar">&times;</button>
            </div>
            ${message ? `<div class="notification-message">${message}</div>` : ''}
            ${!persistent ? '<div class="notification-progress"></div>' : ''}
        `;

        const closeBtn = notification.querySelector('.notification-close');
        closeBtn.addEventListener('click', () => this.hide(id));

        this.container.appendChild(notification);
        this.notifications.set(id, notification);

        setTimeout(() => notification.classList.add('show'), 50);

        if (!persistent && duration > 0) {
            setTimeout(() => this.hide(id), duration);
        }

        return id;
    }

    hide(id) {
        const notification = this.notifications.get(id);
        if (notification) {
            notification.classList.add('hide');
            setTimeout(() => {
                if (notification.parentNode) {
                    notification.parentNode.removeChild(notification);
                }
                this.notifications.delete(id);
            }, 400);
        }
    }

    success(title, message, options = {}) {
        return this.show('success', title, message, options);
    }

    error(title, message, options = {}) {
        return this.show('error', title, message, { ...options, persistent: true });
    }

    warning(title, message, options = {}) {
        return this.show('warning', title, message, options);
    }

    info(title, message, options = {}) {
        return this.show('info', title, message, options);
    }

    clearAll() {
        Array.from(this.notifications.keys()).forEach(id => this.hide(id));
    }
}

// Instância global do gerenciador de notificações
const notify = new NotificationManager();