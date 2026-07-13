import React from "react";
import { NavLink } from "react-router-dom";

import FirebaseAuth from "react-firebaseui/FirebaseAuth";

function translateFirebaseUiLabels() {
  const replacements = {
    "Sign in with email": "使用邮箱登录",
    Email: "邮箱",
    NEXT: "下一步",
    Next: "下一步",
    Back: "返回",
    Cancel: "取消",
    "Enter your email": "请输入邮箱",
  };

  document.querySelectorAll(".firebaseui-container *").forEach((node) => {
    const text = node.textContent && node.textContent.trim();
    if (replacements[text] && node.children.length === 0) {
      node.textContent = replacements[text];
    }
    if (node.placeholder && replacements[node.placeholder]) {
      node.placeholder = replacements[node.placeholder];
    }
    if (node.getAttribute("aria-label") && replacements[node.getAttribute("aria-label")]) {
      node.setAttribute("aria-label", replacements[node.getAttribute("aria-label")]);
    }
  });
}

class SignInScreen extends React.Component {
  // The component's Local state.
  state = {
    isSignedIn: false, // Local signed-in state.
    expanded: true,
  };

  // Configure FirebaseUI.
  uiConfig = {
    // Popup signin flow rather than redirect flow.
    signInFlow: "redirect",
    signInOptions: [
      // firebase.auth.GoogleAuthProvider.PROVIDER_ID,
      // {
      //   provider: firebase.auth.EmailAuthProvider.PROVIDER_ID,
      //   requireDisplayName: false,
      // },
      {
        provider: firebase.auth.EmailAuthProvider.PROVIDER_ID,
        signInMethod: firebase.auth.EmailAuthProvider.EMAIL_LINK_SIGN_IN_METHOD,
        emailLinkSignIn: function () {
          return {
            // Additional state showPromo=1234 can be retrieved from URL on
            // sign-in completion in signInSuccess callback by checking
            // window.location.href.
            url: "https://sandspiel.club/browse",
            // Custom FDL domain.
            // Always true for email link sign-in.
            handleCodeInApp: true,
          };
        },
      },
      // firebase.auth.FacebookAuthProvider.PROVIDER_ID
    ],
    callbacks: {
      // Avoid redirects after sign-in.
      signInSuccessWithAuthResult: () => false,
    },
  };

  // Listen to the Firebase Auth state and set the local state.
  componentDidMount() {
    this.setState({ isSignedIn: !!firebase.auth().currentUser });

    this.unregisterAuthObserver = firebase
      .auth()
      .onAuthStateChanged((user) => this.setState({ isSignedIn: !!user }));
    this.translateFirebaseTimer = window.setInterval(translateFirebaseUiLabels, 250);
  }

  // Make sure we un-register Firebase observers when the component unmounts.
  componentWillUnmount() {
    this.unregisterAuthObserver();
    window.clearInterval(this.translateFirebaseTimer);
  }

  render() {
    if (!this.state.isSignedIn) {
      if (!this.state.expanded) {
        return (
          <span>
            <p>
              请先{" "}
              <button onClick={() => this.setState({ expanded: true })}>
                登录
              </button>{" "}
              后再投票！{" "}
            </p>
            <span style={{ display: "none" }}>
              {/* gross hack for completing login */}
              <FirebaseAuth
                uiConfig={this.uiConfig}
                FirebaseAuth={firebase.auth()}
              />
            </span>
          </span>
        );
      } else {
        return (
          <>
            登录后可以发布和投票！
            <FirebaseAuth
              uiConfig={this.uiConfig}
              firebaseAuth={firebase.auth()}
            />
          </>
        );
      }
    }
    let { currentUser } = firebase.auth();
    // console.log(currentUser);
    return (
      <div>
        <div style={{ display: "flex", alignItems: "center" }}>
          <NavLink
            style={{ flexGrow: 0, margin: "0 4px", fontSize: "19px" }}
            to={{
              pathname: "/browse/search/",
              search: `?user=${currentUser.uid}`,
            }}
          >
            [发布历史]
          </NavLink>
          {!currentUser.emailVerified &&
            `请验证邮箱 ${currentUser.email} 后再投票！`}
          <button
            style={{ flexGrow: 0, margin: "0 10px" }}
            onClick={() => {
              if (window.confirm("确定要退出登录吗？")) {
                firebase.auth().signOut();
              }
            }}
          >
            退出登录
          </button>
        </div>
      </div>
    );
  }
}

export default SignInScreen;
