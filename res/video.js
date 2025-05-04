'use strict';


const CAPTIONS_KEY           = 'c';
const FULLSCREEN_KEY         = 'f';
const PICTURE_IN_PICTURE_KEY = 'p';
const PLAY_KEY               = ' ';
const SEEK_BACKWARD_KEY      = 'ArrowLeft';
const SEEK_FORWARD_KEY       = 'ArrowRight';

/** Time to idle until hiding the controls */
const IDLE_WAIT_MS = 2000;
/** Time to wait until showing a tooltip */
const TOOLTIP_WAIT_MS = 1000;

/** Seconds to seek with arrow keys */
const ARROW_SEEK_SECONDS = 5;


/** @type {HTMLVideoElement} */
const video = document.getElementById('video');

/** @type {HTMLDivElement} */
const controls = document.getElementById('controls');

/** @type {HTMLButtonElement} */
const play_pause_button = document.getElementById('play_pause');
/** @type {HTMLButtonElement} */
const captions_button = document.getElementById('captions');
/** @type {HTMLButtonElement} */
const picture_in_picture_button = document.getElementById('picture_in_picture');
/** @type {HTMLButtonElement} */
const download_button = document.getElementById('download');
/** @type {HTMLButtonElement} */
const fullscreen_button = document.getElementById('fullscreen');

/** @type {SVGElement} */
const play_icon = document.getElementById('play');
/** @type {SVGElement} */
const pause_icon = document.getElementById('pause');
/** @type {SVGElement} */
const captions_on_icon = document.getElementById('captions_on');
/** @type {SVGElement} */
const captions_off_icon = document.getElementById('captions_off');
/** @type {SVGElement} */
const sound_on_icon = document.getElementById('sound_on');
/** @type {SVGElement} */
const sound_off_icon = document.getElementById('sound_off');
/** @type {SVGElement} */
const enter_fullscreen_icon = document.getElementById('enter_fullscreen');
/** @type {SVGElement} */
const exit_fullscreen_icon = document.getElementById('exit_fullscreen');

/** @type {HTMLDivElement} */
const times_and_progress = document.getElementById('times_and_progress');
/** @type {HTMLInputElement} */
const progress = document.getElementById('progress');
/** @type {HTMLSpanElement} */
const current_time_span = document.getElementById('current_time');
/** @type {HTMLSpanElement} */
const total_time_span = document.getElementById('total_time');

/** @type {HTMLDialogElement} */
const dialog = document.getElementById('dialog');
/** @type {HTMLParagraphElement} */
const dialog_message = document.getElementById('dialog_message');


/** Where the video duration was previously in seconds */
let previous_seconds = -1;

/** Idle timeout ID */
let idle_timer = 0;
/** Tooltip timeout ID */
let tooltip_timer = 0;

/**
 * Where the stylus/finger pushed down, to compare with lifting it up
 * @type {EventTarget | null}
 */
let clicked_down_target = null;


/**
 * Handle keys as video playback shortcuts
 * @param {KeyboardEvent} event
 */
function handle_key(event)
{
	if (event.altKey || event.ctrlKey || event.shiftKey)
		return;

	switch (event.key) {
		case CAPTIONS_KEY:
			if (event.repeat)
				break;
			event.preventDefault();
			toggle_captions();
			break;
		case FULLSCREEN_KEY:
			if (event.repeat)
				break;
			event.preventDefault();
			toggle_fullscreen();
			break;
		case PICTURE_IN_PICTURE_KEY:
			if (event.repeat)
				break;
			event.preventDefault();
			toggle_picture_in_picture();
			break;
		case PLAY_KEY:
			if (event.repeat)
				break;
			event.preventDefault();
			toggle_play();
			break;
		case SEEK_BACKWARD_KEY:
			event.preventDefault();
			seek_small(false);
			break;
		case SEEK_FORWARD_KEY:
			event.preventDefault();
			seek_small(true);
			break;
		default:
			break;
	}
}


/**
 * Toggle play/pause and hide controls if playing
 * @param {PointerEvent | undefined} event
 */
function handle_play_pointer_up(event)
{
	if (event.target !== clicked_down_target) {
		clicked_down_target = null;
		return;
	}
	clicked_down_target = null;

	toggle_play();

	if (event.pointerType === 'touch') {
		if (video.paused)
			hide_controls(false);
		else
			start_hiding_controls();
	}
}


/**
 * Show the seek dot and time when the mouse hovers
 * @param {MouseEvent} event
 */
function handle_seek_hover(event)
{
	if (isNaN(video.duration)) {
		set_tooltip(progress, 'Seek');
		return;
	}
	const normalized_x = event.offsetX / event.target.clientWidth;
	const seconds = video.duration * normalized_x;
	const time = to_time(Math.floor(seconds), true);
	set_tooltip(progress, `Seek to ${time}`);
}


/**
 * Show a popup if the video fails
 * @param {string | Event} event
 */
function handle_video_error(event)
{
	dialog_message.innerText = 'Failed to play the video';
	dialog.close();
	dialog.showModal();
	if (typeof event === 'string')
		console.error(event);
	else if (video.error)
		console.error(video.error.message);
}


/**
 * Toggle hiding controls if touched, toggle play/pause if clicked
 * @param {PointerEvent} event
 */
function handle_video_pointer_up(event)
{
	if (event.target !== clicked_down_target) {
		clicked_down_target = null;
		return;
	}
	clicked_down_target = null;

	// Skip if not left click
	if (event.button !== 0)
		return;

	event.preventDefault();

	if (event.pointerType === 'touch') {
		if (document.body.classList.contains('hidden_controls')) {
			hide_controls(false);
			start_hiding_controls();
		}
		else {
			hide_controls(true);
		}
	}
	else {
		toggle_play();
	}
}


/**
 * Start hiding the controls only if the mouse moved a sigficant amount
 * @param {PointerEvent} event
 */
function handle_video_pointer_move_on(event)
{
	if (event.pointerType === 'touch')
		return;

	if (document.body.classList.contains('hidden_controls')) {
		// Show the controls for significant mouse movements
		const MIN = 4;
		if (Math.abs(event.movementX) > MIN || Math.abs(event.movementY) > MIN)
			hide_controls(false);
	}
	else {
		// Restart the timer to hide the controls
		stop_hiding_controls();
		start_hiding_controls();
	}
}


/** When toggling fullscreen, correct the button and hide controls */
function handle_window_change(e)
{
	update_fullscreen_button();
	hide_controls(true);
}


/** Hide the controls at the bottom */
function hide_controls(hide=true)
{
	if (hide) {
		stop_hiding_controls();
		document.body.classList.add('hidden_controls');
	}
	else {
		document.body.classList.remove('hidden_controls');
	}
}


/** Stop showing the custom tooltip text */
function hide_tooltip()
{
	clearTimeout(tooltip_timer);
	const tooltip = document.getElementById('tooltip');
	if (tooltip)
		tooltip.classList.add('hidden');
}


/**
 * Remember the clicked down target
 * @param {PointerEvent} event
 */
function remember_target(event)
{
	clicked_down_target = event.target;
}


/**
 * Seek the video a small amount on keypress
 * @param {boolean} forward
 */
function seek_small(forward)
{
	video.currentTime += forward ? ARROW_SEEK_SECONDS : -ARROW_SEEK_SECONDS;
}


/**
 * Seek the video to the new slider value
 */
function seek_specific()
{
	clicked_down_target = null;
	video.currentTime = video.duration * parseInt(progress.value) / parseInt(progress.max);
}


/**
 * Set the tooltip text (pretty title attribute alternative)
 * @param {HTMLElement} element
 * @param {string} text
 */
function set_tooltip(element, text)
{
	element.ariaLabel = text;
	element.onpointerenter = show_tooltip;
	element.onpointerleave = hide_tooltip;
	if (element.type === 'range')
		element.onpointermove = show_tooltip;
}


/**
 * Start showing the custom tooltip text
 * @param {PointerEvent} event
 */
function show_tooltip(event)
{
	if (!(event.target instanceof Element))
		return;

	// Get or make it
	let tooltip = document.getElementById('tooltip');
	if (!tooltip) {
		tooltip = document.createElement('div');
		tooltip.id = 'tooltip';
		tooltip.className = 'hidden';
		document.body.appendChild(tooltip);
	}

	const is_range = event.target.type === 'range';

	// Set the text and calculate the tooltip position
	const rect = event.target.getBoundingClientRect();
	let x = 0;
	if (is_range) {
		x = event.clientX;
		tooltip.classList.add('range');
	}
	else {
		x = (rect.left + rect.right) / 2;
		tooltip.classList.remove('range');
	}
	tooltip.innerText = event.target.ariaLabel;
	tooltip.style.left = `${x}px`;
	tooltip.style.top = `${rect.top}px`;

	// Show the tooltip
	if (!is_range) {
		tooltip_timer = setTimeout(show_tooltip_delayed, TOOLTIP_WAIT_MS);
		return;
	}
	tooltip.classList.remove('hidden');
}


function show_tooltip_delayed()
{
	const tooltip = document.getElementById('tooltip');
	if (tooltip)
		tooltip.classList.remove('hidden');
}


/** Start downloading the video and any subtitles */
function start_download()
{
	const urls = [video.src];
	for (const track of document.getElementsByTagName('track'))
		urls.push(track.src);

	for (const url of urls) {
		const a = document.createElement('a');
		a.download = '';
		a.href = url;
		a.click();
		a.remove();
	}
}


/** Restart the countdown that hides the controls */
function start_hiding_controls()
{
	idle_timer = setTimeout(hide_controls, IDLE_WAIT_MS);
}


/** Stop the countdown that hides the controls */
function stop_hiding_controls()
{
	clearTimeout(idle_timer);
}


/**
 * Convert total seconds to a time string
 * @param {number} seconds
 * @param {boolean} show_hours
 */
function to_time(seconds, show_hours)
{
	let minutes = Math.floor(seconds / 60);
	let hours = Math.floor(minutes / 60);
	minutes -= hours * 60;
	seconds -= hours * 3600;
	seconds -= minutes * 60;
	return show_hours
		? `${hours}:${two_digits(minutes)}:${two_digits(seconds)}`
		: `${minutes}:${two_digits(seconds)}`;
}


/** Cycle through captions or no captions */
function toggle_captions()
{
	if (!video.textTracks.length)
		return;

	for (let i = 0; i < video.textTracks.length; i++) {
		// Skip the disabled text track
		if (video.textTracks[i].mode !== 'showing')
			continue;
		// Disable the current text track
		video.textTracks[i].mode = 'disabled';
		// Get the next text track, otherwise stop to disable them all
		const next = video.textTracks[i + 1];
		if (!next)
			return update_captions_button();
		// Enable the next text track and stop
		next.mode = 'showing';
		return update_captions_button();
	}

	// Enable the first text track because they were all disabled
	video.textTracks[0].mode = 'showing';
	update_captions_button();
}


/** Toggle between fullscreen landscape or not fullscreen */
function toggle_fullscreen()
{
	if (document.fullscreenElement) {
		document.exitFullscreen();
	}
	else {
		document.body.requestFullscreen();
		screen.orientation.lock('landscape').catch(() => {});
	}
}


/** Play or pause */
function toggle_play()
{
	if (video.paused)
		video.play();
	else
		video.pause();
}


/** Toggle between PiP video window and the normal video display */
function toggle_picture_in_picture()
{
	if (document.pictureInPictureElement)
		document.exitPictureInPicture();
	else
		video.requestPictureInPicture();
}


/**
 * Number to double digit string
 * @param {number} number
 */
function two_digits(number)
{
	return String(number).padStart(2, '0');
}


/**
 * Show whether captions are on and the selected language
 */
function update_captions_button()
{
	let language = '';
	for (const track of video.textTracks) {
		if (track.mode === 'showing') {
			language = track.label;
			break;
		}
	}
	if (video.textTracks.length === 0) {
		captions_button.classList.add('hidden');
	}
	else if (language) {
		set_tooltip(captions_button, `${language} Captions - C`);
		captions_off_icon.classList.add('hidden');
		captions_on_icon.classList.remove('hidden');
		captions_button.classList.remove('hidden');
	}
	else {
		set_tooltip(captions_button, 'Captions Off - C');
		captions_on_icon.classList.add('hidden');
		captions_off_icon.classList.remove('hidden');
		captions_button.classList.remove('hidden');
	}
}


/** Show whether the video is paused */
function update_play_pause_button()
{
	if (video.paused) {
		set_tooltip(play_pause_button, 'Play - Space');
		pause_icon.classList.add('hidden');
		play_icon.classList.remove('hidden');
	}
	else {
		set_tooltip(play_pause_button, 'Pause - Space');
		play_icon.classList.add('hidden');
		pause_icon.classList.remove('hidden');
	}
}


/** Show whether the video is fullscreen */
function update_fullscreen_button()
{
	if (document.fullscreenElement) {
		set_tooltip(fullscreen_button, 'Exit Fullscreen - F or Esc');
		enter_fullscreen_icon.classList.add('hidden');
		exit_fullscreen_icon.classList.remove('hidden');
	}
	else {
		set_tooltip(fullscreen_button, 'Fullscreen - F');
		exit_fullscreen_icon.classList.add('hidden');
		enter_fullscreen_icon.classList.remove('hidden');
	}
}


/** Move the video progress bar */
function update_progress()
{
	if (clicked_down_target === progress)
		return;
	const current_seconds = Math.floor(video.currentTime);
	const total_seconds = Math.floor(video.duration);
	if (current_seconds === previous_seconds || isNaN(total_seconds))
		return;
	previous_seconds = current_seconds;
	progress.value = video.currentTime / video.duration * parseInt(progress.max);
	const show_hours = total_seconds >= 3600;
	current_time_span.innerText = to_time(current_seconds, show_hours);
	total_time_span.innerText = to_time(total_seconds, show_hours);
	if (total_seconds < 600) // 10:00 => 60 * 10
		document.documentElement.style.setProperty('--time_em', 'var(--time_3_digits_em)');
	else if (total_seconds < 3600) // 1:00:00 => 60 * 60
		document.documentElement.style.setProperty('--time_em', 'var(--time_4_digits_em)');
	else
		document.documentElement.style.setProperty('--time_em', 'var(--time_5_digits_em)');
	times_and_progress.classList.remove('invisible');
}


/** Set event handlers, show controls, and hide unavailable buttons */
function main()
{
	document.onkeydown = handle_key;
	document.onmouseleave = hide_controls;
	window.onresize = handle_window_change;

	video.controls = false;
	video.onpointermove = handle_video_pointer_move_on;
	video.onpointerdown = remember_target;
	video.onpointerup = handle_video_pointer_up;
	video.onplay = update_play_pause_button;
	video.onpause = update_play_pause_button;
	video.ontimeupdate = update_progress;
	video.onerror = handle_video_error;

	controls.onpointerenter = stop_hiding_controls;

	progress.onchange = seek_specific;
	progress.onmousemove = handle_seek_hover;
	progress.onpointerdown = remember_target;
	play_pause_button.onpointerdown = remember_target;
	play_pause_button.onpointerup = handle_play_pointer_up;
	captions_button.onclick = toggle_captions;
	download_button.onclick = start_download;
	fullscreen_button.onclick = toggle_fullscreen;

	controls.classList.remove('hidden');

	if (video.requestPictureInPicture)
		picture_in_picture_button.onclick = toggle_picture_in_picture;
	else
		picture_in_picture_button.classList.add('hidden');

	update_play_pause_button();
	update_captions_button();
	update_progress();
	start_hiding_controls();

	for (const element of document.querySelectorAll('[title]')) {
		set_tooltip(element, element.title);
		element.removeAttribute('title');
	}
}


main();
