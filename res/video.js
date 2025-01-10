'use strict';


const CAPTIONS_KEY           = 'c';
const FULLSCREEN_KEY         = 'f';
const PICTURE_IN_PICTURE_KEY = 'p';
const PLAY_KEY               = ' ';
const SEEK_BACKWARD_KEY      = 'ArrowLeft';
const SEEK_FORWARD_KEY       = 'ArrowRight';

/** Time to idle until hiding the controls */
const IDLE_TIME_MS = 2000;

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
const progress = document.getElementById('progress');
/** @type {HTMLDivElement} */
const progress_track = document.getElementById('progress_track');
/** @type {HTMLDivElement} */
const times_and_progress = document.getElementById('times_and_progress');
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
 * Remember the clicked down target
 * @param {PointerEvent} event
 */
function handle_play_pointer_down(event)
{
	clicked_down_target = event.target;
}


/**
 * Show the seek dot and time when the mouse hovers
 * @param {MouseEvent} event
 */
function handle_seek_hover(event)
{
	const style = document.documentElement.style;
	if (isNaN(video.duration)) {
		style.setProperty('--progress_hover_percent', '0%');
		progress_track.title = 'Seek';
		return;
	}
	const normalized_x = event.offsetX / event.target.clientWidth;
	style.setProperty('--progress_hover_percent', `${normalized_x * 100}%`);
	const seconds = video.duration * normalized_x;
	const time = to_time(Math.floor(seconds), true);
	progress_track.title = `Seek to ${time}`;
}


/** Show a popup if the video fails */
function handle_video_error()
{
	dialog_message.innerText = 'Failed to play the video';
	dialog.close();
	dialog.showModal();
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

	const MIN = 2;
	if (Math.abs(event.movementX) < MIN && Math.abs(event.movementY) < MIN)
		return;

	if (document.body.classList.contains('hidden_controls')) {
		hide_controls(false);
	}
	else {
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


/**
 * Seek the video a small amount on keypress
 * @param {boolean} forward
 */
function seek_small(forward)
{
	video.currentTime += forward ? ARROW_SEEK_SECONDS : -ARROW_SEEK_SECONDS;
}


/**
 * Seek the video to the clicked point
 * @param {PointerEvent} event
 */
function seek_specific(event)
{
	video.currentTime = video.duration * event.offsetX / event.target.clientWidth;
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
	idle_timer = setTimeout(hide_controls, IDLE_TIME_MS);
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
		captions_button.title = `${language} Captions - C`;
		captions_off_icon.classList.add('hidden');
		captions_on_icon.classList.remove('hidden');
		captions_button.classList.remove('hidden');
	}
	else {
		captions_button.title = 'Captions Off - C';
		captions_on_icon.classList.add('hidden');
		captions_off_icon.classList.remove('hidden');
		captions_button.classList.remove('hidden');
	}
}


/** Show whether the video is paused */
function update_play_pause_button()
{
	if (video.paused) {
		play_pause_button.title = 'Play - Space';
		pause_icon.classList.add('hidden');
		play_icon.classList.remove('hidden');
	}
	else {
		play_pause_button.title = 'Pause - Space';
		play_icon.classList.add('hidden');
		pause_icon.classList.remove('hidden');
	}
}


/** Show whether the video is fullscreen */
function update_fullscreen_button()
{
	if (document.fullscreenElement) {
		fullscreen_button.title = 'Exit Fullscreen - F or Esc';
		enter_fullscreen_icon.classList.add('hidden');
		exit_fullscreen_icon.classList.remove('hidden');
	}
	else {
		fullscreen_button.title = 'Fullscreen - F';
		exit_fullscreen_icon.classList.add('hidden');
		enter_fullscreen_icon.classList.remove('hidden');
	}
}


/** Move the video progress bar */
function update_progress()
{
	const current_seconds = Math.floor(video.currentTime);
	const total_seconds = Math.floor(video.duration);
	if (current_seconds === previous_seconds || isNaN(total_seconds))
		return;
	previous_seconds = current_seconds;
	progress.style.width = `${video.currentTime / video.duration * 100}%`
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
	video.onpointerdown = handle_play_pointer_down;
	video.onpointerup = handle_video_pointer_up;
	video.onplay = update_play_pause_button;
	video.onpause = update_play_pause_button;
	video.ontimeupdate = update_progress;
	video.onerror = handle_video_error;

	controls.onpointerenter = stop_hiding_controls;

	progress_track.onclick = seek_specific;
	progress_track.onmousemove = handle_seek_hover;
	play_pause_button.onpointerdown = handle_play_pointer_down;
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
}


main();
