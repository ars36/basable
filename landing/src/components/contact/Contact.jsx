import React, {useState} from "react";
import "./Contact.css";
import contactpic from "./images/contactpic.svg";
import starticon from "./images/icon.svg";

function Contact() {
    const [form, setForm] = useState({
        fullname: "",
        email: "",
        message: "",
    });

    const handleChange = (event) => {
        const { name, value } = event.target.value;
        setForm({ ...form, [name]: value });
    };

    const handleSubmit = (event) => {
        event.preventDefault();
        console.log("Form Data:", form);
        setForm({
            fullname: "",
            email: "",
            message: "",
        })
    }

  return (
    <div>
      <section>
        <div className="contact-container">
          <div className="contact-left-side">
            <div className="top-contact-part">
              <div className="started">
                <div className="icon-container">
                  {" "}
                  <img className="logo-icon" src={starticon} alt="logo" />
                </div>
                <p className="started-text">Get Started</p>
              </div>
              <div className="contact-head-text">
                <h1>Contact Us</h1>
                <p>
                  Please fill this form to send us any questions or concerns you
                  have.
                </p>
              </div>
            </div>
            <div className="contact-bottom-part">
              <form onSubmit={handleSubmit}>
                <div className="input-label">
                  <p className="full-name">Full Name</p>
                  <input
                    type="text"
                    name="name"
                    value={form.fullname}
                    onChange={handleChange}
                  ></input>
                </div>

                <div className="input-label">
                  <p className="email">Email</p>
                  <input
                    type="email"
                    name="email"
                    value={form.email}
                    onChange={handleChange}
                  ></input>
                </div>

                <div className="message-label">
                  <p className="message">Your Message</p>
                  <textarea
                    name="message"
                    value={form.message}
                    onChange={handleChange}
                  ></textarea>
                </div>
              </form>
              <button
                className="submit-btn"
                type="submit"
                onClick={handleSubmit}
              >
                Submit
              </button>
            </div>
          </div>
          <div className="contact-right-side">
            <img className="contact-photo" src={contactpic} alt="call" />
          </div>
        </div>
      </section>
    </div>
  );
}

export default Contact;
