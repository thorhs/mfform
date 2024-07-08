<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/thorhs/mfform">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a>

<h3 align="center">mfform</h3>

  <p align="center">
    A simple input form simulating mainframe input forms, think 3270.
    <br />
    <!--- <a href="https://github.com/thorhs/mfform"><strong>Explore the docs »</strong></a>
    <br />
    <br / --->
    <a href="https://github.com/thorhs/mfform">View Demo</a>
    ·
    <a href="https://github.com/thorhs/mfform/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    ·
    <a href="https://github.com/thorhs/mfform/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

[![Product Name Screen Shot][product-screenshot]](https://example.com)

mfframe is a small tool I created for gather bits of data from the user.
I've recently been playing with [Hercules][Hercules-url], the mainframe emulator,
as well as having a long standing facinations with them.
So, I did what anyone would do, I created a small utility that presents the user
with a 3270-like dialog asking for input.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

* [![Rust][Rustlang.org]][Rust-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

Start by creating a screen.mfform with the inputs requested (see below),
then run the program by calling mfform.

### Prerequisites

For now you need the rust compiler:
* rustup
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Installation

   ```sh
   cargo install --git https://github.com/thorhs/mfform

   ```

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

### Configuration file

An example configuration file is below:
```
LABEL 8 2 USER ===>
LABEL 4 4 PASSWORD ===>
LABEL 6 6 NUMBER ===>
LABEL 5 8 DEFAULT ===>

INPUT    18 2 8 username
PASSWORD 18 4 8 password
NUMBER   18 6 8 number
INPUT    18 8 8 default awesome!

SELECT username id1 First!
SELECT username id2 Second
SELECT username id3 Third
SELECT username id4 Fourth
```

LABEL lines are made up of the LABEL keyword, followed by the x,y coordinates for the label, and the value.  Everthing after the y coordinate is used as the text, there is no need to quote the string.

INPUT lines start with the same x,y coordinates followed by the input field length and the field name.  An optional default value can follow the field name.

PASSWORD lines work just as the INPUT lines, except the input value is masked on screen.  Please note that the value will be in plain text in the program output.

NUMBER lines work like INPUT lines, except the only accept numbers.

SELECT lines have a field name, item ID and item text.  This enables F4 for the particular input field and adds the id/text combo as a possible item to select.

### Using the utility

Once you have mfform running the following keyboard shortcuts are available:

* Enter - Submits the input form, causing the program to print the field values in a name=value format and exiting.
* Esc - Aborts the input form, nothing gets written to stdout and the program exits.
* F4 - For input fields that have SELECTs, will trigger a 'popup' form allowing the user to select an item for use as value.
* Tab/Shift+Tab - Next/Previus input field.
* Arrow keys - Move around on the screen.
* Backspace - When on an input field will remove the previous character and shift the remainder to the left.
* Del - When on an input field will remove the current character and shift the remainder to the left.
* Ctrl-D - Will enable debug output below the form, not really usefull for end users.
* Ctrl-C - Should always abort the form and exit cleanly to shell.
* Any other character - Overwrite the current character when in an input field.  There is no inser functionality yet.  Any unicode 'should' be supported.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap

- [ ] Insert functionality
- [ ] Possibly LUA, or other, embedded script for populating SELECTs

See the [open issues](https://github.com/thorhs/mfform/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Your Name - [@thorhs](https://twitter.com/thorhs) - toti@toti.is

Project Link: [https://github.com/thorhs/mfform](https://github.com/thorhs/mfform)

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/thorhs/mfform.svg?style=for-the-badge
[contributors-url]: https://github.com/thorhs/mfform/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/thorhs/mfform.svg?style=for-the-badge
[forks-url]: https://github.com/thorhs/mfform/network/members
[stars-shield]: https://img.shields.io/github/stars/thorhs/mfform.svg?style=for-the-badge
[stars-url]: https://github.com/thorhs/mfform/stargazers
[issues-shield]: https://img.shields.io/github/issues/thorhs/mfform.svg?style=for-the-badge
[issues-url]: https://github.com/thorhs/mfform/issues
[license-shield]: https://img.shields.io/github/license/thorhs/mfform.svg?style=for-the-badge
[license-url]: https://github.com/thorhs/mfform/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/thorhs
[product-screenshot]: images/screenshot.png
[Hercules-url]: http://www.hercules-390.org
[Rustlang.org]: https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust
[Rust-url]: https://rustlang.org
