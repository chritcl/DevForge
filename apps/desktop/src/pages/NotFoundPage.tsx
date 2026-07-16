import { Link } from "react-router";

export function NotFoundPage() {
  return (
    <div className="not-found-page">
      <h1>页面不存在</h1>
      <p>请检查地址是否正确。</p>
      <Link to="/" className="error-link">
        返回首页
      </Link>
    </div>
  );
}
