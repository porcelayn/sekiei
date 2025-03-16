function copyCode(button) {
    const codeBlock = button.closest('.code-block').querySelector('code');
    const codeLines = codeBlock.querySelectorAll('.code-line');
    const codeText = Array.from(codeLines)
        .map(line => line.textContent)
        .join('\n');
    
    navigator.clipboard.writeText(codeText)
        .then(() => console.log('Copied:\n' + codeText))
        .catch(err => console.error('Failed to copy:', err));
}

function setTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('theme', theme);
}

function toggleTheme() {
    const currentTheme = localStorage.getItem('theme') || 'light';
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    
    if (window.loadGiscus) {
        window.loadGiscus();
    }
}

function initializeTheme() {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
        setTheme(savedTheme);
    } else {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        setTheme(prefersDark ? 'dark' : 'light');
    }
}

document.addEventListener('DOMContentLoaded', () => {
    initializeTheme();
    
    const toggleButton = document.getElementById('toggle-theme');
    if (toggleButton) {
        toggleButton.addEventListener('click', toggleTheme);
    }
});