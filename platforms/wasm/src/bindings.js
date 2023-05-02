let canvas = document.getElementById('canvas');

let callback = e => { }
export function set_callback(c) {
    callback = c;
}

export function request_redraw() {
    window.requestAnimationFrame(e => callback('Redraw'));
}

window.addEventListener('resize', e => {
    let dpi = window.devicePixelRatio;
    let width = canvas.clientWidth * dpi;
    let height = canvas.clientHeight * dpi;
    canvas.width = width;
    canvas.height = height;
    callback({'Resize': {'width': width, 'height': height}});
})

window.addEventListener('beforeunload', e => {
    callback('Unloading');
})

canvas.addEventListener('mousemove', e => {
    let dpi = window.devicePixelRatio;
    callback({'MouseMove': {'x': e.x * dpi, 'y': e.y * dpi}});
    e.preventDefault();
});
canvas.addEventListener('mousedown', e => {
    callback('MouseDown');
    e.preventDefault();
});
canvas.addEventListener('mouseup', e => {
    callback('MouseUp');
    e.preventDefault();
});
canvas.addEventListener('wheel', e => {
    switch (e.deltaMode) {
        case e.DOM_DELTA_PIXEL:
            callback({'MouseWheel': {'amt': -e.deltaY / 100.0}});
            break;
        case e.DOM_DELTA_LINE:
            callback({'MouseWheel': {'amt': -e.deltaY / 4.0}});
            break;
    }
    e.preventDefault();
});
let touchHandler = e => {
    let dpi = window.devicePixelRatio;
    for (let i = 0; i < e.changedTouches.length; i++) {
        let touch = e.changedTouches[i];
        callback({
            'Touch': {
                'phase': e.type,
                'x': touch.clientX * dpi,
                'y': touch.clientY * dpi,
                'id': touch.identifier
            }
        });
    }
    e.preventDefault();
};
canvas.addEventListener('touchstart', touchHandler);
canvas.addEventListener('touchmove', touchHandler);
canvas.addEventListener('touchend', touchHandler);
canvas.addEventListener('touchcancel', touchHandler);

