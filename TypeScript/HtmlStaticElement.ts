
class HtmlStaticElement<T> {
    value: T;
    el: HTMLElement;

    constructor(el: HTMLElement) {
        this.el = el;
    }

    public update(value: T, toString: (value: T) => string) {

        if (this.value === undefined || this.value != value) {
            this.value = value;
            this.el.innerHTML = toString(value);
        }
    }

}