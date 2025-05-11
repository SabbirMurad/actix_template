document.body.addEventListener('pointerdown', (e) => {
    const rippleWrapper = document.createElement('div');
    rippleWrapper.classList.add('ripple-wrapper');
    const ripple = document.createElement('div');
    ripple.classList.add('ripple');
    ripple.style.left = `${e.clientX - 50}px`;
    ripple.style.top = `${e.clientY - 50}px`;
    rippleWrapper.appendChild(ripple);
    document.body.appendChild(rippleWrapper);
    setTimeout(() => rippleWrapper.remove(), 600); // Match animation duration
});