
export function set_callback(callback) {
    let canvas = document.getElementById('canvas');
    canvas.addEventListener('mousemove', e => {
        callback({'MouseMove': {'x': e.x, 'y': e.y}});
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
                callback({'MouseWheel': {'amt': -e.deltaY / 2.0}});
                break;
        }
        e.preventDefault();
    });
}
