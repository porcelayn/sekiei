function copyCode(button) {
    const codeBlock = button.closest('.code-block').querySelector('code');
    const codeLines = codeBlock.querySelectorAll('.code-line');
    const codeText = Array.from(codeLines)
        .map(line => line.textContent)
        .join('\n'); 
    navigator.clipboard.writeText(codeText).then(() => {
        console.log('Copied:\n' + codeText);
    }).catch((err) => {
        console.error('Failed to copy: ', err);
    });
}