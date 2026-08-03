import { createGlobalStyle } from 'antd-style';

export const GlobalStyle = createGlobalStyle`
  * {
    box-sizing: border-box;
  }

  html, body, #root {
    margin: 0;
    padding: 0;
    height: 100%;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC',
      'Hiragino Sans GB', 'Microsoft YaHei', 'Helvetica Neue', Helvetica, Arial,
      sans-serif;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    background: #f5f7fa;
    color: rgba(0, 0, 0, 0.88);
  }

  #root {
    display: flex;
    flex-direction: column;
  }

  a {
    color: #1677ff;
    text-decoration: none;
  }

  a:hover {
    color: #4096ff;
  }
`;
