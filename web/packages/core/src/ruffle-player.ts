import { Ruffle } from "../pkg/ruffle_web";

import { load_ruffle } from "./load-ruffle";
import { ruffleShadowTemplate } from "./shadow-template";
import { lookup_element } from "./register-element";

export const FLASH_MIMETYPE = "application/x-shockwave-flash";
export const FUTURESPLASH_MIMETYPE = "application/futuresplash";
export const FLASH7_AND_8_MIMETYPE = "application/x-shockwave-flash2-preview";
export const FLASH_MOVIE_MIMETYPE = "application/vnd.adobe.flash-movie";
export const FLASH_ACTIVEX_CLASSID =
    "clsid:D27CDB6E-AE6D-11cf-96B8-444553540000";

const DIMENSION_REGEX = /^\s*(\d+(\.\d+)?(%)?)/;

declare global {
    interface Window {
        RufflePlayer: any;
    }

    interface Document {
        webkitFullscreenEnabled?: boolean;
        webkitFullscreenElement?: HTMLElement;
        webkitCancelFullScreen?: () => void;
    }

    interface HTMLElement {
        webkitRequestFullScreen?: () => void;
    }
}

function sanitize_parameters(
    parameters:
        | (URLSearchParams | string | Record<string, string>)
        | undefined
        | null
): Record<string, string> {
    if (parameters === null || parameters === undefined) {
        return {};
    }
    if (!(parameters instanceof URLSearchParams)) {
        parameters = new URLSearchParams(parameters);
    }
    const output: Record<string, string> = {};

    for (const [key, value] of parameters) {
        // Every value must be type of string
        output[key] = value.toString();
    }

    return output;
}

export class RufflePlayer extends HTMLElement {
    private shadow: ShadowRoot;
    private dynamic_styles: HTMLStyleElement;
    private container: HTMLElement;
    private play_button: HTMLElement;
    private right_click_menu: HTMLElement;
    private instance: Ruffle | null;
    allow_script_access: boolean;
    private _trace_observer: ((message: string) => void) | null;
    private Ruffle: Promise<{ new (...args: any[]): Ruffle }>;
    private panicked = false;

    constructor() {
        super();

        this.shadow = this.attachShadow({ mode: "closed" });
        this.shadow.appendChild(ruffleShadowTemplate.content.cloneNode(true));

        this.dynamic_styles = <HTMLStyleElement>(
            this.shadow.getElementById("dynamic_styles")
        );
        this.container = this.shadow.getElementById("container")!;
        this.play_button = this.shadow.getElementById("play_button")!;
        if (this.play_button) {
            this.play_button.addEventListener(
                "click",
                this.play_button_clicked.bind(this)
            );
        }
        this.right_click_menu = this.shadow.getElementById("right_click_menu")!;

        this.addEventListener(
            "contextmenu",
            this.open_right_click_menu.bind(this)
        );

        window.addEventListener("click", this.hide_right_click_menu.bind(this));

        this.instance = null;
        this.allow_script_access = false;
        this._trace_observer = null;

        this.Ruffle = load_ruffle();

        return this;
    }

    connectedCallback() {
        this.update_styles();
    }

    static get observedAttributes() {
        return ["width", "height"];
    }

    attributeChangedCallback(
        name: string,
        oldValue: string | undefined,
        newValue: string | undefined
    ) {
        if (name === "width" || name === "height") {
            this.update_styles();
        }
    }

    disconnectedCallback() {
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
            console.log("Ruffle instance destroyed.");
        }
    }

    update_styles() {
        if (this.dynamic_styles.sheet) {
            if (this.dynamic_styles.sheet.rules) {
                for (
                    let i = 0;
                    i < this.dynamic_styles.sheet.rules.length;
                    i++
                ) {
                    this.dynamic_styles.sheet.deleteRule(i);
                }
            }

            const widthAttr = this.attributes.getNamedItem("width");
            if (widthAttr !== undefined && widthAttr !== null) {
                const width = RufflePlayer.html_dimension_to_css_dimension(
                    widthAttr.value
                );
                if (width !== null) {
                    this.dynamic_styles.sheet.insertRule(
                        `:host { width: ${width}; }`
                    );
                }
            }

            const heightAttr = this.attributes.getNamedItem("height");
            if (heightAttr !== undefined && heightAttr !== null) {
                const height = RufflePlayer.html_dimension_to_css_dimension(
                    heightAttr.value
                );
                if (height !== null) {
                    this.dynamic_styles.sheet.insertRule(
                        `:host { height: ${height}; }`
                    );
                }
            }
        }
    }

    /**
     * Determine if this element is the fallback content of another Ruffle
     * player.
     *
     * This heurustic assumes Ruffle objects will never use their fallback
     * content. If this changes, then this code also needs to change.
     */
    is_unused_fallback_object() {
        let parent = this.parentNode;
        const element = lookup_element("ruffle-object");

        if (element !== null) {
            while (parent != document && parent != null) {
                if (parent.nodeName === element.name) {
                    return true;
                }

                parent = parent.parentNode;
            }
        }

        return false;
    }

    /**
     * Ensure a fresh Ruffle instance is ready on this player before continuing.
     *
     * @throws Any exceptions generated by loading Ruffle Core will be logged
     * and passed on.
     */
    async ensure_fresh_instance() {
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
            console.log("Ruffle instance destroyed.");
        }

        const Ruffle = await this.Ruffle.catch((e) => {
            console.error("Serious error loading Ruffle: " + e);

            // Serious duck typing. In error conditions, let's not make assumptions.
            const message =
                e && e.message ? String(e.message).toLowerCase() : "";
            if (message.indexOf("mime") >= 0) {
                this.panicked = true;
                this.container.innerHTML = `
                    <div id="panic">
                        <div id="panic-title">Something went wrong :(</div>
                        <div id="panic-body">
                            <p>Ruffle has encountered a major issue whilst trying to initialize.</p>
                            <p>This web server is either not serving ".wasm" files with the correct MIME type, or the file cannot be found.</p>
                            <p>If you are the server administrator, please consult the Ruffle wiki for help.</p>
                        </div>
                        <div id="panic-footer">
                            <ul>
                                <li><a href="https://github.com/ruffle-rs/ruffle/wiki/Using-Ruffle#configure-wasm-mime-type">view Ruffle wiki</a></li>
                            </ul>
                        </div>
                    </div>
                `;
            }
            throw e;
        });

        this.instance = new Ruffle(
            this.container,
            this,
            this.allow_script_access
        );
        console.log("New Ruffle instance created.");
    }

    /**
     * Load a movie into this Ruffle Player instance by URL.
     *
     * Any existing movie will be immediately stopped, while the new movie's
     * load happens asynchronously. There is currently no way to await the file
     * being loaded, or any errors that happen loading it.
     *
     * @param {String} url The URL to stream.
     * @param {URLSearchParams|String|Object} [parameters] The parameters (also known as "flashvars") to load the movie with.
     * If it's a string, it will be decoded into an object.
     * If it's an object, every key and value must be a String.
     * These parameters will be merged onto any found in the query portion of the swf URL.
     */
    async stream_swf_url(
        url: string,
        parameters:
            | URLSearchParams
            | string
            | Record<string, string>
            | undefined
            | null
    ) {
        //TODO: Actually stream files...
        try {
            if (this.isConnected && !this.is_unused_fallback_object()) {
                console.log("Loading SWF file " + url);

                await this.ensure_fresh_instance();
                parameters = {
                    ...sanitize_parameters(url.substring(url.indexOf("?"))),
                    ...sanitize_parameters(parameters),
                };
                this.instance!.stream_from(url, parameters);

                if (this.play_button) {
                    this.play_button.style.display = "block";
                }
            } else {
                console.warn(
                    "Ignoring attempt to play a disconnected or suspended Ruffle element"
                );
            }
        } catch (err) {
            console.error("Serious error occurred loading SWF file: " + err);
            this.panic(err);
            throw err;
        }
    }

    play_button_clicked() {
        if (this.instance) {
            this.instance.play();
            if (this.play_button) {
                this.play_button.style.display = "none";
            }
        }
    }

    /**
     * Checks if this player is allowed to be fullscreen by the browser.
     *
     * @returns True if you may call [[enterFullscreen]].
     */
    get fullscreenEnabled(): boolean {
        return !!(
            document.fullscreenEnabled || document.webkitFullscreenEnabled
        );
    }

    /**
     * Checks if this player is currently fullscreen inside the browser.
     *
     * @returns True if it is fullscreen.
     */
    get isFullscreen(): boolean {
        return (
            (document.fullscreenElement || document.webkitFullscreenElement) ===
            this
        );
    }

    /**
     * Requests the browser to make this player fullscreen.
     *
     * This is not guaranteed to succeed, please check [[fullscreenEnabled]] first.
     */
    enterFullscreen(): void {
        if (this.requestFullscreen) {
            this.requestFullscreen();
        } else if (this.webkitRequestFullScreen) {
            this.webkitRequestFullScreen();
        }
    }

    /**
     * Requests the browser to no longer make this player fullscreen.
     */
    exitFullscreen(): void {
        if (document.exitFullscreen) {
            document.exitFullscreen();
        } else if (document.webkitCancelFullScreen) {
            document.webkitCancelFullScreen();
        }
    }

    right_click_menu_items() {
        const items = [];
        if (this.fullscreenEnabled) {
            if (this.isFullscreen) {
                items.push({
                    text: "Exit fullscreen",
                    onClick: this.exitFullscreen.bind(this),
                });
            } else {
                items.push({
                    text: "Enter fullscreen",
                    onClick: this.enterFullscreen.bind(this),
                });
            }
        }
        items.push({
            text: `Ruffle ${
                __CHANNEL__ === "nightly"
                    ? `nightly ${__COMMIT_DATE__}`
                    : window.RufflePlayer.version
            }`,
            onClick() {
                window.open("https://ruffle.rs/", "_blank");
            },
        });
        return items;
    }

    open_right_click_menu(e: MouseEvent) {
        e.preventDefault();

        // Clear all `right_click_menu` items.
        while (this.right_click_menu.firstChild) {
            this.right_click_menu.removeChild(this.right_click_menu.firstChild);
        }

        // Populate `right_click_menu` items.
        for (const { text, onClick } of this.right_click_menu_items()) {
            const element = document.createElement("li");
            element.className = "menu_item active";
            element.textContent = text;
            element.addEventListener("click", onClick);
            this.right_click_menu.appendChild(element);
        }

        // Place `right_click_menu` in the top-left corner, so
        // its `clientWidth` and `clientHeight` are not clamped.
        this.right_click_menu.style.left = "0";
        this.right_click_menu.style.top = "0";
        this.right_click_menu.style.display = "block";

        const rect = this.getBoundingClientRect();
        const x = e.clientX - rect.x;
        const y = e.clientY - rect.y;
        const maxX = rect.width - this.right_click_menu.clientWidth - 1;
        const maxY = rect.height - this.right_click_menu.clientHeight - 1;

        this.right_click_menu.style.left = Math.floor(Math.min(x, maxX)) + "px";
        this.right_click_menu.style.top = Math.floor(Math.min(y, maxY)) + "px";
    }

    hide_right_click_menu() {
        this.right_click_menu.style.display = "none";
    }

    pause() {
        if (this.instance) {
            this.instance.pause();
            if (this.play_button) {
                this.play_button.style.display = "block";
            }
        }
    }

    /**
     * Load a movie's data into this Ruffle Player instance.
     *
     * Any existing movie will be immediately stopped, and the new movie's data
     * placed into a fresh Stage on the same stack.
     *
     * Please note that by doing this, no URL information will be provided to
     * the movie being loaded.
     *
     * @param {Iterable<number>} data The data to stream.
     * @param {URLSearchParams|String|Object} [parameters] The parameters (also known as "flashvars") to load the movie with.
     * If it's a string, it will be decoded into an object.
     * If it's an object, every key and value must be a String.
     */
    async play_swf_data(
        data: Iterable<number>,
        parameters:
            | URLSearchParams
            | string
            | Record<string, string>
            | undefined
            | null
    ) {
        try {
            if (this.isConnected && !this.is_unused_fallback_object()) {
                console.log("Got SWF data");

                await this.ensure_fresh_instance();
                this.instance!.load_data(
                    new Uint8Array(data),
                    sanitize_parameters(parameters)
                );
                console.log("New Ruffle instance created.");

                if (this.play_button) {
                    this.play_button.style.display = "block";
                }
            } else {
                console.warn(
                    "Ignoring attempt to play a disconnected or suspended Ruffle element"
                );
            }
        } catch (err) {
            console.error("Serious error occurred loading SWF file: " + err);
            this.panic(err);
            throw err;
        }
    }

    /*
     * Copies attributes and children from another element to this player element.
     * Used by the polyfill elements, RuffleObject and RuffleEmbed.
     */
    copy_element(elem: HTMLElement) {
        if (elem) {
            for (let i = 0; i < elem.attributes.length; i++) {
                const attrib = elem.attributes[i];
                if (attrib.specified) {
                    // Issue 468: Chrome "Click to Active Flash" box stomps on title attribute
                    if (
                        attrib.name === "title" &&
                        attrib.value === "Adobe Flash Player"
                    ) {
                        continue;
                    }

                    try {
                        this.setAttribute(attrib.name, attrib.value);
                    } catch (err) {
                        // The embed may have invalid attributes, so handle these gracefully.
                        console.warn(
                            `Unable to set attribute ${attrib.name} on Ruffle instance`
                        );
                    }
                }
            }

            for (const node of Array.from(elem.children)) {
                this.appendChild(node);
            }
        }
    }

    /*
     * Converts a dimension attribute on an HTML embed/object element to a valid CSS dimension.
     * HTML element dimensions are unitless, but can also be percentages.
     * Add a 'px' unit unless the value is a percentage.
     * Returns null if this is not a valid dimension.
     */
    static html_dimension_to_css_dimension(attribute: string) {
        if (attribute) {
            const match = attribute.match(DIMENSION_REGEX);
            if (match) {
                let out = match[1];
                if (!match[3]) {
                    // Unitless -- add px for CSS.
                    out += "px";
                }
                return out;
            }
        }
        return null;
    }

    /*
     * When a movie presents a new callback through `ExternalInterface.addCallback`,
     * we are informed so that we can expose the method on any relevant DOM element.
     */
    on_callback_available(name: string) {
        const instance = this.instance;
        (<any>this)[name] = (...args: any[]) => {
            return instance?.call_exposed_callback(name, args);
        };
    }

    /*
     * Sets a trace observer on this flash player.
     *
     * The observer will be called, as a function, for each message that the playing movie will "trace" (output).
     */
    set trace_observer(observer: ((message: string) => void) | null) {
        this.instance?.set_trace_observer(observer);
    }

    /*
     * Panics this specific player, forcefully destroying all resources and displays an error message to the user.
     *
     * This should be called when something went absolutely, incredibly and disastrously wrong and there is no chance
     * of recovery.
     *
     * Ruffle will attempt to isolate all damage to this specific player instance, but no guarantees can be made if there
     * was a core issue which triggered the panic. If Ruffle is unable to isolate the cause to a specific player, then
     * all players will panic and Ruffle will become "poisoned" - no more players will run on this page until it is
     * reloaded fresh.
     */
    panic(error: Error | null) {
        if (this.panicked) {
            // Only show the first major error, not any repeats - they aren't as important
            return;
        }
        this.panicked = true;

        // Clears out any existing content (ie play button or canvas) and replaces it with the error screen
        this.container.innerHTML = `
            <div id="panic">
                <div id="panic-title">Something went wrong :(</div>
                <div id="panic-body">
                    <p>Ruffle has encountered a major issue whilst trying to display this Flash content.</p>
                    <p>This isn't supposed to happen, so we'd really appreciate if you could file a bug!</p>
                </div>
                <div id="panic-footer">
                    <ul>
                        <li><a href="https://github.com/ruffle-rs/ruffle/issues/new">report bug</a></li>
                        <li><a href="#" id="panic-view-details">view error details</a></li>
                    </ul>
                </div>
            </div>
        `;
        (<HTMLLinkElement>(
            this.container.querySelector("#panic-view-details")
        )).onclick = () => {
            let error_text = "# Error Info\n";

            if (error instanceof Error) {
                error_text += `Error name: ${error.name}\n`;
                error_text += `Error message: ${error.message}\n`;
                if (error.stack) {
                    error_text += `Error stack:\n\`\`\`\n${error.stack}\n\`\`\`\n`;
                }
            } else {
                error_text += `Error: ${error}\n`;
            }

            error_text += "\n# Player Info\n";
            error_text += this.debug_player_info();

            error_text += "\n# Page Info\n";
            error_text += `Page URL: ${document.location.href}\n`;

            error_text += "\n# Browser Info\n";
            error_text += `Useragent: ${window.navigator.userAgent}\n`;
            error_text += `OS: ${window.navigator.platform}\n`;

            error_text += "\n# Ruffle Info\n";
            error_text += `Ruffle version: ${window.RufflePlayer.version}\n`;
            error_text += `Ruffle source: ${window.RufflePlayer.name}\n`;
            this.container.querySelector(
                "#panic-body"
            )!.innerHTML = `<textarea>${error_text}</textarea>`;
            return false;
        };

        // Do this last, just in case it causes any cascading issues.
        if (this.instance) {
            this.instance.destroy();
            this.instance = null;
        }
    }

    debug_player_info() {
        return `Allows script access: ${this.allow_script_access}\n`;
    }
}

/*
 * Returns whether the given filename ends in an "swf" extension.
 */
export function is_swf_filename(filename: string | null) {
    return (
        filename &&
        (filename.search(/\.swf(?:[?#]|$)/i) >= 0 ||
            filename.search(/\.spl(?:[?#]|$)/i) >= 0)
    );
}
