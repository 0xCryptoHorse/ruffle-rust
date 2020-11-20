/**
 * The shadow template which is used to fill the actual Ruffle player element
 * on the page.
 */
export const ruffleShadowTemplate = document.createElement("template");
ruffleShadowTemplate.innerHTML = `
    <style>
        :host {
            display: inline-block;
            /* Default width/height; this will get overridden by user styles/attributes */
            width: 550px;
            height: 400px;
            touch-action: none;
            user-select: none;
            -webkit-user-select: none;
            -webkit-tap-highlight-color: transparent;
            position: relative;
        }

        #container {
            position: relative;
            width: 100%;
            height: 100%;
            overflow: hidden;
        }

        #container canvas {
            width: 100%;
            height: 100%;
        }

        #play_button {
            position: absolute;
            width: 100%;
            height: 100%;
            cursor: pointer;
            display: none;
        }

        #play_button .icon {
            position: absolute;
            top: 50%;
            left: 50%;
            width: 90%;
            height: 90%;
            max-width: 500px;
            max-height: 500px;
            transform: translate(-50%, -50%);
        }

        #play_button:hover .icon {
            filter: brightness(1.3);
        }
        
        #unmute_overlay {
            position: absolute;
            width: 100%;
            height: 100%;
            cursor: pointer;
            display: none;
        }
        
        #unmute_overlay .background {
            position: absolute;
            width: 100%;
            height: 100%;
            background-color: #000;
            opacity: 0.8;
        }
        
        #unmute_overlay .icon {
            position: relative;
            top: 50%;
            left: 50%;
            width: 90%;
            height: 90%;
            max-width: 500px;
            max-height: 500px;
            transform: translate(-50%, -50%);
            opacity: 0.6;
        }

        #panic {
            position: absolute;
            width: 100%;
            height: 100%;
            /* Inverted colours from play button! */
            background: linear-gradient(180deg, rgba(253,58,64,1) 0%, rgba(253,161,56,1) 100%);
            color: #000;
        }

        #panic a {
            color: #37528C;
        }

        #panic-title {
            margin-top: 30px;
            text-align: center;
            font-size: 42px;
            font-weight: bold;
        }

        #panic-body {
            text-align: center;
            font-size: 20px;
            position: absolute;
            top: 100px;
            bottom: 80px;
            left: 50px;
            right: 50px;
        }

        #panic-body textarea {
            width: 100%;
            height: 100%;
        }

        #panic-footer {
            position: absolute;
            bottom: 30px;
            text-align: center;
            font-size: 20px;
            width: 100%;
        }

        #panic ul {
            margin: 35px 0 0 0;
            padding: 0;
            max-width: 100%;
            display: flex;
            list-style-type: none;
            justify-content: center;
            align-items: center;
        }

        #panic li {
            padding: 10px 50px;
        }

        #right_click_menu {
            background-color: #37528c;
            color: #FFAD33;
            border-radius: 5px;
            position: absolute;
            list-style: none;
            padding: 0;
            margin: 0;
        }

        #right_click_menu .menu_item {
            padding: 5px 10px;
        }

        #right_click_menu .menu_separator {
            padding: 5px 5px;
        }

        #right_click_menu .active {
            cursor: pointer;
            color: #FFAD33;
        }

        #right_click_menu .disabled {
            cursor: default;
            color: #94672f;
        }

        #right_click_menu .active:hover {
            background-color: #184778;
        }

        #right_click_menu hr {
            color: #FFAD33;
        }

        #right_click_menu > :first-child {
            border-top-right-radius: 5px;
            border-top-left-radius: 5px;
        }

        #right_click_menu > :last-child {
            border-bottom-right-radius: 5px;
            border-bottom-left-radius: 5px;
        }
    </style>
    <style id="dynamic_styles"></style>

    <div id="container">
        <div id="play_button"><div class="icon"><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" preserveAspectRatio="xMidYMid" viewBox="0 0 250 250" style="width:100%;height:100%;"><defs><linearGradient id="a" gradientUnits="userSpaceOnUse" x1="125" y1="0" x2="125" y2="250" spreadMethod="pad"><stop offset="0%" stop-color="#FDA138"/><stop offset="100%" stop-color="#FD3A40"/></linearGradient><g id="b"><path fill="url(#a)" d="M250 125q0-52-37-88-36-37-88-37T37 37Q0 73 0 125t37 88q36 37 88 37t88-37q37-36 37-88M87 195V55l100 70-100 70z"/><path fill="#FFF" d="M87 55v140l100-70L87 55z"/></g></defs><use xlink:href="#b"/></svg></div></div>
        <div id="unmute_overlay"><div class="background"></div><div class="icon"><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" preserveAspectRatio="xMidYMid" viewBox="0 0 512 584" style="width:100%;height:100%;scale:0.8;"><path fill="#FFF" stroke="#FFF" d="m457.941 256 47.029-47.029c9.372-9.373 9.372-24.568 0-33.941-9.373-9.373-24.568-9.373-33.941 0l-47.029 47.029-47.029-47.029c-9.373-9.373-24.568-9.373-33.941 0-9.372 9.373-9.372 24.568 0 33.941l47.029 47.029-47.029 47.029c-9.372 9.373-9.372 24.568 0 33.941 4.686 4.687 10.827 7.03 16.97 7.03s12.284-2.343 16.971-7.029l47.029-47.03 47.029 47.029c4.687 4.687 10.828 7.03 16.971 7.03s12.284-2.343 16.971-7.029c9.372-9.373 9.372-24.568 0-33.941z"/><path fill="#FFF" stroke="#FFF" d="m99 160h-55c-24.301 0-44 19.699-44 44v104c0 24.301 19.699 44 44 44h55c2.761 0 5-2.239 5-5v-182c0-2.761-2.239-5-5-5z"/><path fill="#FFF" stroke="#FFF" d="m280 56h-24c-5.269 0-10.392 1.734-14.578 4.935l-103.459 79.116c-1.237.946-1.963 2.414-1.963 3.972v223.955c0 1.557.726 3.026 1.963 3.972l103.459 79.115c4.186 3.201 9.309 4.936 14.579 4.936h23.999c13.255 0 24-10.745 24-24v-352.001c0-13.255-10.745-24-24-24z"/><text x="256" y="560" text-anchor="middle" style="font-size:60px;fill:#FFF;stroke:#FFF;">Click to unmute</text></svg></div></div>
    </div>

    <ul id="right_click_menu" style="display: none"></ul>
`;
