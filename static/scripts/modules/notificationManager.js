export default class NotificationManager {
    constructor() {
        this.container = document.getElementById('notifications-container');
        if (!this.container) {
            console.error('Notification container not found!');
            return;
        }
        this.notifications = new Map();
        this.notificationCounter = 0;
    }

    show(type, title, message, options = {}) {
        const id = `notification-${++this.notificationCounter}`;
        const duration = options.duration || 5000;
        const persistent = options.persistent || false;

        const icons = { success: '✅', error: '❌', warning: '⚠️', info: 'ℹ️' };

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

        notification.querySelector('.notification-close').addEventListener('click', () => this.hide(id));
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
                notification.remove();
                this.notifications.delete(id);
            }, 400);
        }
    }

    success(title, message, options = {}) { this.show('success', title, message, options); }
    error(title, message, options = {}) { this.show('error', title, message, { ...options, persistent: true }); }
    warning(title, message, options = {}) { this.show('warning', title, message, options); }
    info(title, message, options = {}) { this.show('info', title, message, options); }
}